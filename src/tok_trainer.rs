use std::cmp::min;
// Use STD only, avoid external dependencies unless it speeds up by 3x!!!!!
use std::fs::File;
use std::io::Write;
use std::ops::AddAssign;
use std::panic;
use std::collections::BTreeMap;
// Don't use those garbage collection stuffs unless you really need it!
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use crate::TokenizerParameters;


#[derive(Debug, PartialEq)]
struct ConcatenatedBytes {
	// A structure that replaces `Vec<Vec<u8>>` by storing data in a concatenated `Vec<u8>`
	// This will avoid the overhead of multiple `Vec<u8>` allocations and providing memory-contiguous storage
	// But profiled with VTune, this didn't seem to speed up in noticeable way
	data: Vec<u8>,
	bounds: Vec<std::ops::Range<usize>>,
}

impl ConcatenatedBytes {
	fn new(data: Vec<u8>, bounds: Vec<std::ops::Range<usize>>) -> Self {
		ConcatenatedBytes { data, bounds }
	}

	// Don't even bother with methods because they're obfuscated
}

// It'll let `sum_byte_pair_encoding()` accept i16 and i32 inputs
trait SumBPE: Copy + AddAssign<Self> + Into<i32> {}
impl SumBPE for i16 {}
impl SumBPE for i32 {}


fn train_unigram_bytes(byte_vec: &ConcatenatedBytes, dropout: Option<u32>, pre_keyed_map: Option<&BTreeMap<Vec<u8>, i16>>) -> BTreeMap<Vec<u8>, i16> {
	fn drop_keys(mut loop_count: u32, dropout: u32, mut counter: BTreeMap<Vec<u8>, i16>) -> (u32, BTreeMap<Vec<u8>, i16>) {
		loop_count += 1;
		if loop_count % dropout == 0 {
			// Dropping keys that are less than 2 saves 85% on average memory
			// But it dropped keys that might have been more than 1 count late because they were far away
			counter.retain(|_, &mut count| count > 1);
		}
		(loop_count, counter)
	}
	// This function takes up 50% of CPU time on average of the whole program
	// Pre-keying will result in much fewer memcmp
	let mut counter = pre_keyed_map.unwrap_or(&BTreeMap::new()).clone(); // BTreeMap is faster than HashMap; profiled with VTune
	let dropout = dropout.unwrap_or(0xf_ffff);

	let mut loop_count = 0;
	for bound in &byte_vec.bounds {
		for i in bound.start..bound.end {
			for j in i..bound.end {
				// Memory write and read reduction idea: Compress slices with a JPG tokenizer
				let slice = byte_vec.data[i..j + 1].to_vec();
				// Unlock memory bandwidth ​with AVX2: Replace Vec<u8> with __m256i
				*counter.entry(slice).or_insert(0) += 1;
				(loop_count, counter) = drop_keys(loop_count, dropout, counter);
			}
		}
	}
	counter.retain(|_, &mut count| count > 1);
	counter
}

fn numerical_grade_encodable(byte_vec: &ConcatenatedBytes, counter: &BTreeMap<Vec<u8>, i16>) -> BTreeMap<Vec<u8>, i16> {
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

fn all_subbyte(byte_arr: &[u8], pattern: &[u8], pre_allocate_len: Option<usize>) -> Vec<usize> {
	let mut pattern_matches = Vec::with_capacity(pre_allocate_len.unwrap_or(0));
	for (index, slice) in byte_arr.windows(pattern.len()).enumerate() {
		if pattern == slice {
			pattern_matches.push(index);
		}
	}
	pattern_matches
}

fn generate_cutoff_by_pattern(byte_vec: &ConcatenatedBytes, pattern: &[u8]) -> Vec<std::ops::Range<usize>> {
	fn map_usize_to_ranges(indices: &[usize], length: usize) -> Vec<std::ops::Range<usize>> {
		indices.iter().map(|&i| i..i + length).collect()
	}

	fn merge_ranges(indices: &[std::ops::Range<usize>]) -> Vec<std::ops::Range<usize>> {
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

fn rebuild_2d_byte_vec(cutoff: &[std::ops::Range<usize>], byte: &ConcatenatedBytes) -> ConcatenatedBytes {
	let mut new_byte = ConcatenatedBytes::new(vec![], vec![]);
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

fn greedy_bpe_encode(byte: &[u8]) -> BTreeMap<Vec<u8>, i16> {
	// This method will encode the byte pair encoding using `count * sub.len() - count * byte_size`
	// greedy scoring method without testing every single combinations so it'd be fast
	// But, who knows if this will result in optimal size
	let mut byte_vec = ConcatenatedBytes::new(byte.to_vec(), vec![0..byte.len()]);
	let mut tokenizer_model = BTreeMap::new();

	while !byte_vec.data.is_empty() {
		let counter = train_unigram_bytes(&byte_vec, None, None);
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

		// Memory read and write optimization idea: Don't resize the vector and just rebuild the bounds only
		// Maybe it's possible to convert `ConcatenatedBytes` from `Vec<u8>` to `&[u8]` to speed this up
		byte_vec = rebuild_2d_byte_vec(&cutoff, &byte_vec);
	}
	tokenizer_model
}

fn sum_byte_pair_encoding<S: SumBPE>(tokenizer: &BTreeMap<Vec<u8>, i32>, stats: &BTreeMap<Vec<u8>, S>) -> BTreeMap<Vec<u8>, i32> {
	// Increasing trainable range idea: Create a bf16 imitation to store higher values in the BPE
	// But this will increase the update resistance as the number goes up due to quantized rounding errors,
	// and slowing down the program by not using the built-in ASM instructions but software emulating the bf16 type
	stats.into_iter().fold(tokenizer.clone(), |mut summed_model, (key, value)| {
		*summed_model.entry(key.to_vec()).or_insert(0) += (*value).into();
		summed_model
	})
}

fn train_tokenizer_rayon_multi_threaded(param: &TokenizerParameters) -> BTreeMap<Vec<u8>, i32> {
	// Don't use hyper-threading as it uses twice as much memory for a 5% improvement only
	let num_threads = if param.multi_threaded.unwrap() == 0 {
		let cores = num_cpus::get_physical();
		if param.has_info() { println!("Info: multi_threaded is 0: Auto assigning {} thread(s)", cores); }
		cores
	} else {
		param.multi_threaded.unwrap()
	};
	if param.has_debug() { println!("Debug: Will use {} thread(s)", num_threads); }
	let pool = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();

	let chunk_length = param.trainer_chk_bytes.unwrap();
	let bin_vec = param.bin_dat.clone().unwrap();
	let chunks: Vec<&[u8]> = bin_vec.chunks(chunk_length).collect();
	let group_size = (chunks.len() + num_threads - 1) / num_threads;
	let groups: Vec<&[&[u8]]> = chunks.chunks(group_size).collect();
	if param.has_debug() { println!("Debug: Chunk length: {} * {}", chunk_length, chunks.len()); }

	// Completed; Multi-cores idea: Sum the model within threads,
	// the memory usage should be limited to the number of threads rather than a vector,
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

fn train_tokenizer_single_thread(param: &TokenizerParameters) -> BTreeMap<Vec<u8>, i32> {
	let mut tokenizer_model = BTreeMap::new();
	let bin_dat = param.bin_dat.clone().unwrap();
	let chunks = bin_dat.chunks(param.trainer_chk_bytes.unwrap());

	if param.has_debug() { println!("Debug: Will use single thread only on {} chunk(s)", chunks.len()); }
	for chunk in chunks {
		let line = chunk.to_vec();
		tokenizer_model = sum_byte_pair_encoding(&tokenizer_model, &greedy_bpe_encode(&line));
	}
	tokenizer_model
}

pub fn train_tokenizer(param: &mut TokenizerParameters) -> BTreeMap<Vec<u8>, i32> {
	param.trainer_chk_bytes = match param.trainer_chk_bytes {
		Some(chunk_length) => Some(chunk_length),
		None => {
			let default_chunk_length = min(16, param.bin_dat.clone().unwrap().len());
			if param.has_info() { println!("Info: chunk_length is None; assigning chunk(s) with length {}", default_chunk_length); }
			Some(default_chunk_length)
		}
	};
	let tokenizer_model;
	if param.multi_threaded != None {
		tokenizer_model = train_tokenizer_rayon_multi_threaded(param)
	} else {
		tokenizer_model = train_tokenizer_single_thread(param)
	}
	tokenizer_model
		.into_iter()
		.map(|(k, v)| (k.clone(), v - k.len() as i32))
		.collect()
}

pub fn entry(tok_trainer_args: &mut TokenizerParameters) {
	let bin_dat = tok_trainer_args.bin_dat.as_ref().unwrap();
	if tok_trainer_args.has_info() { println!("Info: File byte size: {}", bin_dat.len()); }
	let result = train_tokenizer(tok_trainer_args);
	if tok_trainer_args.has_lengthy() { println!("Lengthy: greedy_bpe_encode: {:?}, length: {}", result, result.len()); }

	let mut file = File::create("output.vocab.txt").expect("create failed");
	for (byte_vec, score) in result {
		writeln!(file, "{:?}\t{}", byte_vec, score).expect("write failed");
	}
}

#[cfg(test)]
mod tests {
	use std::{collections::BTreeMap, io::{BufReader, Read}};
	use crate::{tok_trainer::*, debug_enum};

	#[test]
	fn test_train_unigram_bytes() {
		let byte_vec = ConcatenatedBytes::new(vec![
				1, 2, 3, 1, 2, 3, 1, 2,
				3, 1, 2, 3, 1, 2, 3, 1,
			],
			vec![0..8, 8..16]
		);

		let test_dropout = 0x3fff;
		let counter = train_unigram_bytes(&byte_vec, Some(test_dropout), None);

		assert_eq!(*counter.get(&vec![1, 2]).unwrap(), 5);
		assert_eq!(*counter.get(&vec![3, 1]).unwrap(), 5);
		assert_eq!(*counter.get(&vec![1, 2, 3]).unwrap(), 4);
		assert_eq!(*counter.get(&vec![3, 1, 2]).unwrap(), 4);
	}

	#[test]
	fn test_numerical_grade_encodable() {
		let byte_vec = ConcatenatedBytes::new(vec![
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
		let byte_vec = ConcatenatedBytes::new(vec![
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
		let byte_vec = ConcatenatedBytes::new(vec![
				1, 2, 3, 1, 2, 3, 1, 2,
				3, 1, 2, 3, 1, 2, 3, 1,
			],
			vec![0..8, 8..16]
		);
		let cutoff = vec![6..8, 8..9, 15..16];
		let result = rebuild_2d_byte_vec(&cutoff, &byte_vec);
		assert_eq!(result, ConcatenatedBytes::new(vec![
				1, 2, 3, 1
			],
			vec![0..2, 2..3, 3..4]
		));
	}

	#[test]
	fn test_train_tokenizer() {
		{ // This will test every functions in the trainer to ensure it won't crash
		let result_too_short = train_tokenizer(&mut TokenizerParameters {
			multi_threaded: Some(2),
			dbg_lv: debug_enum::DEBUG,
			bin_dat: Some(b"a".to_vec()),
			bytes_to_read: None,
			trainer_chk_bytes: Some(2),
		});
		let result = train_tokenizer(&mut TokenizerParameters {
			multi_threaded: Some(2),
			dbg_lv: debug_enum::DEBUG,
			bin_dat: Some(b"abcdabcc".to_vec()),
			bytes_to_read: None,
			trainer_chk_bytes: None,
		});

		assert_eq!(result_too_short, BTreeMap::new());
		let mut expected = BTreeMap::new();
		expected.insert(b"abc".to_vec(), 1);
		assert_eq!(result, expected);
		}
		{ // File test
		let file = File::open("pexels-pixabay-302743.jpg").expect("Unable to open file");
		let bytes_to_read = 0xffff_ffff;

		let reader = BufReader::new(file);
		let mut input = vec![];
		reader.take(bytes_to_read).read_to_end(&mut input).expect("Unable to read file");

		let result = train_tokenizer(&mut TokenizerParameters {
			multi_threaded: Some(num_cpus::get_physical()),
			dbg_lv: debug_enum::DEBUG,
			bin_dat: Some(input),
			bytes_to_read: None,
			trainer_chk_bytes: Some(16),
		});
		let mut file = File::create("output.vocab.txt").expect("create failed");
		for (byte_vec, score) in result {
			writeln!(file, "{:?}\t{}", byte_vec, score).expect("write failed");
		}
		}
	}
}
