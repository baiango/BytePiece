use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;


const DEMO_STRING: &[u8] = b"The quick brown fox jumps over the lazy dog.";

#[derive(Debug)]
struct TokCodec {
	model: BTreeMap<Vec<u8>, i32>
}


impl TokCodec {
	fn new(file_path: &str) -> TokCodec {
		let file = File::open(file_path).unwrap();
		let reader = BufReader::new(file);
		let mut model = BTreeMap::new();

		for line in reader.lines() {
			let line = line.unwrap();
			let parts: Vec<&str> = line.split('\t').collect(); // E.g.: ["[0, 0]", "40"]

			// E.g.: "[0, 0]"
			let vector_part = &parts[0][1..parts[0].len()-1]; // E.g.: "0, 0"
			let vector_elements: Vec<u8> = vector_part.split(',')
				.map(|s| s.trim())
				.map(|s| u8::from_str(s).unwrap())
				.collect(); // E.g.: [0, 0]

			let integer_part = i32::from_str(parts[1]).unwrap(); // E.g.: 40

			model.insert(vector_elements, integer_part);
		}

		TokCodec { model }
	}

	fn from_longest_bytes_iter(&self) -> impl Iterator<Item = (Vec<u8>, i32)> + '_ {
		let mut mapped: Vec<Vec<u8>> = self.model.iter().map(|(bytes, _)| bytes.clone()).collect();
		mapped.sort_by_key(|bytes| bytes.len());
		mapped.into_iter().rev().map(move |bytes| (bytes.clone(), self.model[&bytes]))
	}

	fn encode(&self, bytes: &[u8]) {
		// Start with the longest string
		for (bytes, freq) in self.from_longest_bytes_iter() {
			println!("Bytes: {:?}, Length: {}, Frequency: {}", bytes, bytes.len(), freq);
		}
		unimplemented!()
	}

	fn decode(&self) {
		unimplemented!()
	}
}


pub fn demo() {
	let model = TokCodec::new("output.vocab.txt");
	// println!("{:?}", model);
	let tokens = model.encode(DEMO_STRING);
	println!("{:?}", tokens);
}
