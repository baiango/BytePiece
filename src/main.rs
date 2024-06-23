// Use STD only, avoid external dependencies unless it speeds up by 3x!!!!!
use std::fs::File;
use std::io::{Read, BufReader};
use std::io::Write;

use crate::tokenizer_trainer::train_tokenizer;

mod tokenizer_trainer {
	use std::ops::AddAssign;
	use std::panic;
	use std::{collections::BTreeMap, cmp::min};
	use std::sync::{Arc, Mutex};
	use rayon::prelude::*;

	#[derive(Debug, PartialEq)]
	pub struct ConcatenatedByte {
		// A structure that replaces `Vec<Vec<u8>>` by storing data in a concatenated `Vec<u8>`
		// This will avoiding the overhead of multiple `Vec<u8>` allocations and providing memory-contiguous storage
		data: Vec<u8>,
		bounds: Vec<std::ops::Range<usize>>,
	}

	impl ConcatenatedByte {
		pub fn new(data: Vec<u8>, bounds: Vec<std::ops::Range<usize>>) -> Self {
			ConcatenatedByte { data, bounds }
		}
	}

	// It'll let `sum_byte_pair_encoding()` accept i16 and i32 inputs
	pub trait SumBPE: Copy + AddAssign<Self> + Into<i32> {}
	impl SumBPE for i16 {}
	impl SumBPE for i32 {}

	pub fn count_u8_subvectors(byte_vec: &ConcatenatedByte, dropout: Option<u32>, pre_keyed_map: Option<BTreeMap<Vec<u8>, i16>>) -> BTreeMap<Vec<u8>, i16> {
		// This function takes up 50% of CPU time on average of the whole program
		// Pre-keying will result in much fewer memcmp
		let mut counter = pre_keyed_map.unwrap_or(BTreeMap::new()); // BTreeMap is faster than HashMap; profiled with VTune
		let dropout = dropout.unwrap_or(0xf_ffff);

		let mut loop_count = 0;
		for bound in &byte_vec.bounds {
			for i in bound.start..bound.end {
				for j in i..bound.end {
					// Memory write and read reduction idea: Compress slices with a JPG tokenizer
					let slice = byte_vec.data[i..j + 1].to_vec();
					// Unlock memory bandwidth ​with AVX2: Replace Vec<u8> with __m256i
					*counter.entry(slice).or_insert(0) += 1;

					loop_count += 1;
					if loop_count % dropout == 0 {
						// Dropping keys that are less than 2 saves 85% on average memory
						// But it dropped keys that might have been more than 1 count late because they were far away
						counter.retain(|_, &mut count| count > 1);
					}
				}
			}
		}
		counter.retain(|_, &mut count| count > 1);
		counter
	}

	pub fn numerical_grade_encodable(byte_vec: &ConcatenatedByte, counter: &BTreeMap<Vec<u8>, i16>) -> BTreeMap<Vec<u8>, i16> {
		// It returns the `count * sub.len() - count * byte_size` as the score,
		// and assumes that storing tokens take only one byte
		// But, most likely this is not the case but two bytes
		let byte_size = 1;
		let mut scores = BTreeMap::new();
		for sub in counter.keys() {
			scores.insert(sub.to_vec(), 0);
		}
		for range in &byte_vec.bounds {
			for sub in counter.keys() {
				let count = byte_vec.data[range.clone()].windows(sub.len()).filter(|w| w == sub).count() as i16;
				*scores.get_mut(sub).unwrap() += count * sub.len() as i16 - count * byte_size;
			}
		}
		scores
	}

	pub fn all_subbyte(byte_arr: &[u8], pattern: &[u8], pre_allocate_len: Option<usize>) -> Vec<usize> {
		let mut pattern_matches = Vec::with_capacity(pre_allocate_len.unwrap_or(0));
		for (index, slice) in byte_arr.windows(pattern.len()).enumerate() {
			if pattern == slice {
				pattern_matches.push(index);
			}
		}
		pattern_matches
	}

	pub fn map_usize_to_ranges(indices: &[usize], length: usize) -> Vec<std::ops::Range<usize>> {
		indices.iter().map(|&i| i..i + length).collect()
	}

	pub fn merge_ranges(indices: &[std::ops::Range<usize>]) -> Vec<std::ops::Range<usize>> {
		indices.chunks(2).filter_map(|chunk| {
			if chunk.len() < 2 {
				Some(chunk[0].clone())
			} else if chunk[0].end == chunk[1].start {
				Some(chunk[0].start..chunk[1].end)
			} else {
				None
			}
		}).collect()
	}

	fn invert_ranges(merged_ranges: &[std::ops::Range<usize>], byte_index: usize, byte_range_len: usize) -> Vec<std::ops::Range<usize>> {
		let mut inverted_ranges = vec![];

		if merged_ranges[0].start != byte_index {
			inverted_ranges = vec![byte_index..merged_ranges[0].start];
		}

		for range in merged_ranges {
			if range.end != byte_range_len + byte_index {
				inverted_ranges.push(range.end..byte_range_len + byte_index);
			}
		}
		inverted_ranges
	}

	pub fn generate_cutoff_by_pattern(byte_vec: &ConcatenatedByte, pattern: &[u8]) -> Vec<std::ops::Range<usize>> {
		let mut cutoff = vec![];
		let mut byte_index = 0;
		for byte_range in &byte_vec.bounds {
			let inner_vec = &byte_vec.data[byte_range.clone()];

			let subbyte_output = all_subbyte(&inner_vec, pattern, None);
			if subbyte_output.is_empty() {
				continue;
			}

			let ranges: Vec<std::ops::Range<usize>> = map_usize_to_ranges(&subbyte_output, pattern.len())
				.iter()
				.map(|range| range.start + byte_index..range.end + byte_index)
				.collect();

			let merged_ranges = merge_ranges(&ranges);
			if merged_ranges.is_empty() {
				continue;
			}

			let inverted_ranges = invert_ranges(&merged_ranges, byte_index, byte_range.len());

			cutoff.extend(inverted_ranges);
			byte_index += byte_range.len();
		}
		cutoff
	}

	pub fn rebuild_2d_byte_vec(cutoff: &[std::ops::Range<usize>], byte: &ConcatenatedByte) -> ConcatenatedByte {
		let mut new_byte = ConcatenatedByte::new(vec![], vec![]);
		for range in cutoff {
			if range.start != range.end {
				let slice = &byte.data[range.clone()];
				new_byte.data.extend_from_slice(slice);
				new_byte.bounds.push(new_byte.data.len() - slice.len()..new_byte.data.len());
			} else {
				panic!("Maybe generate_cutoff_by_pattern have bugs.\nbyte: {:?}\ncutoff: {:?}", byte, cutoff);
			}
		}
		new_byte
	}

	pub fn greedy_bpe_encode(byte: &[u8]) -> BTreeMap<Vec<u8>, i16> {
		// This method will encode the byte pair encoding using `count * sub.len() - count * byte_size`
		// greedy scoring method so it'd be fast
		// But, who knows if this will result in optimal size
		let mut byte_vec = ConcatenatedByte::new(byte.to_vec(), vec![0..byte.len()]);
		let mut tokenizer_model = BTreeMap::new();

		while !byte_vec.data.is_empty() {
			let counter = count_u8_subvectors(&byte_vec, None, None);
			if counter.is_empty() {
				return tokenizer_model;
			}

			let scores = numerical_grade_encodable(&byte_vec, &counter);

			let (best_subvector, best_score) = scores.iter().max_by_key(|(_, v)| *v).unwrap();

			if 2 >= *best_score {
				return tokenizer_model;
			}

			tokenizer_model.insert(best_subvector.clone(), *best_score);

			let cutoff = generate_cutoff_by_pattern(&byte_vec, best_subvector);
			byte_vec = rebuild_2d_byte_vec(&cutoff, &byte_vec);
		}
		tokenizer_model
	}

	pub fn sum_byte_pair_encoding<S: SumBPE>(tokenizer: &BTreeMap<Vec<u8>, i32>, stats: &BTreeMap<Vec<u8>, S>) -> BTreeMap<Vec<u8>, i32> {
		// Increasing trainable range idea: Create a bf16 imitation to store higher values in the BPE
		// But this will increase the update resistance as the number goes up due to quantized rounding errors,
		// and slowing down the program by not using the built-in ASM instructions but software emulating the bf16 type
		stats.into_iter().fold(tokenizer.clone(), |mut summed_model, (key, value)| {
			*summed_model.entry(key.to_vec()).or_insert(0) += (*value).into();
			summed_model
		})
	}

	pub fn train_tokenizer_rayon_multi_threaded(byte_arr: &[u8], chunk_length: usize) -> BTreeMap<Vec<u8>, i32> {
		// Don't use hyper-threading as it uses twice as much memory for a 5% improvement only
		let num_physical_cores = num_cpus::get_physical();
		println!("Will use {} threads", num_physical_cores);
		let pool = rayon::ThreadPoolBuilder::new().num_threads(num_physical_cores).build().unwrap();

		let chunks: Vec<&[u8]> = byte_arr.chunks(chunk_length).collect();
		let group_size = (chunks.len() + num_physical_cores - 1) / num_physical_cores;
		let groups: Vec<&[&[u8]]> = chunks.chunks(group_size).collect();
		println!("Chunk length: {} * {}", chunk_length, chunks.len());

		// Completed; Multi-cores idea: Sum the model within threads,
		// the memory usage should limited to the amount of threads rathser than a vector,
		// and, it doesn't have to use mutexes to merge the model except at the end
		let tokenizer_model = Arc::new(Mutex::new(BTreeMap::new()));
		pool.install(|| groups.par_iter().for_each(|&group| {
			let local_model = group.into_iter().fold(BTreeMap::new(), |mut local_model, &line| {
				let bpe = greedy_bpe_encode(line);
				local_model = sum_byte_pair_encoding(&local_model, &bpe);
				local_model
			});

			let mut tokenizer_model_guard = tokenizer_model.lock().unwrap();
			*tokenizer_model_guard = sum_byte_pair_encoding(&*tokenizer_model_guard, &local_model);
		}));

		Arc::try_unwrap(tokenizer_model).unwrap().into_inner().unwrap()
	}

	pub fn train_tokenizer_single_thread(byte_arr: &[u8], chunk_length: usize) -> BTreeMap<Vec<u8>, i32> {
		let mut tokenizer_model = BTreeMap::new();
		println!("Will use single thread only");
		for chunk in byte_arr.chunks(chunk_length) {
			let line = chunk.to_vec();
			tokenizer_model = sum_byte_pair_encoding(&tokenizer_model, &greedy_bpe_encode(&line));
		}
		tokenizer_model
	}

	pub fn train_tokenizer(byte_arr: &[u8], chunk_length: Option<usize>, multi_threaded: bool) -> BTreeMap<Vec<u8>, i32> {
		let chunk_length = min(chunk_length.unwrap_or(512), byte_arr.len());
		let tokenizer_model;
		if multi_threaded {
			tokenizer_model = train_tokenizer_rayon_multi_threaded(byte_arr, chunk_length)
		} else {
			tokenizer_model = train_tokenizer_single_thread(byte_arr, chunk_length)
		}
		tokenizer_model
			.into_iter()
			.map(|(k, v)| (k.clone(), v - k.len() as i32))
			.collect()
	}
}


fn main() {
	let file = File::open("pexels-pixabay-302743.jpg").expect("Unable to open file");
	let reader = BufReader::new(file);
	let mut input = vec![];
	reader.take(0xffff_ffff).read_to_end(&mut input).expect("Unable to read file");

	println!("input size: {}", input.len());
	let result = train_tokenizer(&input, Some(16), true);
	println!("greedy_bpe_encode: {:?}, length: {}", result, result.len());

	let mut file = File::create("output.vocab.txt").expect("create failed");
	for (byte_vec, score) in result {
		writeln!(file, "{:?}\t{}", byte_vec, score).expect("write failed");
	}
}


#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;
	use crate::tokenizer_trainer::*;

	#[test]
	pub fn test_count_u8_subvectors() {
		let byte_vec = ConcatenatedByte::new(vec![
				1, 2, 3, 1, 2, 3, 1, 2,
				3, 1, 2, 3, 1, 2, 3, 1,
			],
			vec![0..8, 8..16]
		);

		let counter = count_u8_subvectors(&byte_vec, Some(0x3fff), None);

		assert_eq!(*counter.get(&vec![1, 2]).unwrap(), 5);
		assert_eq!(*counter.get(&vec![3, 1]).unwrap(), 5);
		assert_eq!(*counter.get(&vec![1, 2, 3]).unwrap(), 4);
		assert_eq!(*counter.get(&vec![3, 1, 2]).unwrap(), 4);
	}

	#[test]
	fn test_numerical_grade_encodable() {
		let byte_vec = ConcatenatedByte::new(vec![
				1, 2, 3, 1, 2, 3, 1, 2,
				3, 1, 2, 3, 1, 2, 3, 1,
			],
			vec![0..8, 8..16]
		);
		let mut counter = BTreeMap::new();
		counter.insert(vec![1, 2], 0);
		counter.insert(vec![3, 1], 0);

		let scores = numerical_grade_encodable(&byte_vec, &counter);
		assert_eq!(*scores.get(&vec![1, 2]).unwrap(), 5);
		assert_eq!(*scores.get(&vec![3, 1]).unwrap(), 5);
	}

	#[test]
	fn test_all_substring() {
		let binary: Vec<u8> = ("a".repeat(10000) + "b" + &"a".repeat(5) + "b").bytes().collect();
		let pattern: Vec<u8> = "ab".bytes().collect();
		let result = all_subbyte(&binary, &pattern, Some(5));
		assert_eq!(result, vec![9999, 10005]);
		assert_eq!(all_subbyte(b"qwertyuiop", b"tyu", None), vec![4]);
		assert_eq!(all_subbyte(b"qwertyuiop", b"asd", Some(1000)), vec![]);
	}

	#[test]
	fn test_generate_cutoff_by_pattern() {
		let byte_vec = ConcatenatedByte::new(vec![
				1, 2, 3, 1, 2, 3, 1, 2,
				3, 1, 2, 3, 1, 2, 3, 1,
			],
			vec![0..8, 8..16]
		);
		let pattern = vec![1, 2, 3];
		let result = generate_cutoff_by_pattern(&byte_vec, &pattern);
		assert_eq!(result, vec![6..8, 8..9, 15..16]);
	}

	#[test]
	fn test_rebuild_2d_byte_vec() {
		let byte_vec = ConcatenatedByte::new(vec![
				1, 2, 3, 1, 2, 3, 1, 2,
				3, 1, 2, 3, 1, 2, 3, 1,
			],
			vec![0..8, 8..16]
		);
		let cutoff = vec![6..8, 8..9, 15..16];
		let result = rebuild_2d_byte_vec(&cutoff, &byte_vec);
		assert_eq!(result, ConcatenatedByte::new(vec![
				1, 2, 3, 1
			],
			vec![0..2, 2..3, 3..4]
		));
	}

	#[test]
	fn test_train_tokenizer() {
		let result_too_short = train_tokenizer(&b"a".to_vec(), Some(2), true);
		let result = train_tokenizer(&b"abcdabcc".to_vec(), None, false);

		assert_eq!(result_too_short, BTreeMap::new());
		let mut expected = BTreeMap::new();
		expected.insert(b"abc".to_vec(), 1);
		assert_eq!(result, expected);
	}
}
