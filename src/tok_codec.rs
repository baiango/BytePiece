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
		// Work in progress: Buggy
		unimplemented!();
		let file = File::open(file_path).unwrap();
		let reader = BufReader::new(file);
		let mut model = BTreeMap::new();

		for line in reader.lines() {
			let line = line.unwrap();
			let parts: Vec<&str> = line.split('\t').collect();
			if parts.len() != 2 {
				println!("Error: Expected 2 tab-separated values, found {}", parts.len());
				continue;
			}

			let binary_str = parts[0].trim_matches(['[', ']', ' ']);
			let binary: Vec<u8> = binary_str
				.split(',')
				.map(|x| u8::from_str(x).unwrap_or_else(|_err| {
					println!("Error parsing binary number: {}", x);
					0 // Use 0 as the default value
				}))
				.collect();
			let value: i32 = parts[1].parse().unwrap_or_else(|_err| {
				println!("Error parsing integer value: {}", parts[1]);
				0 // Use 0 as the default value
			});

			model.insert(binary, value);
		}

		TokCodec { model }
	}

	fn encode(&self, bytes: &[u8]) {
		// Start with the longest string
		unimplemented!()
	}

	fn decode(&self) {
		unimplemented!()
	}
}


pub fn demo() {
	let model = TokCodec::new("output.vocab.txt");
	println!("{:?}", model);
	let tokens = model.encode(DEMO_STRING);
	println!("{:?}", tokens);
}
