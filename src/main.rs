mod tok_trainer;
mod tok_codec;
use std::{env, fs::File, path::Path, io::{BufReader, Read}, cmp::min};


mod debug_enum {
#![allow(dead_code)]
	pub const SILENT: u8 = 0;
	pub const ERROR: u8 = 0b1;
	pub const WARN: u8 = 0b10;
	pub const DEBUG: u8 = 0b100;
	pub const INFO: u8 = 0b1000;
	pub const VERBOSE: u8 = 0b1_0000;
	pub const LENGTHY: u8 = 0b10_0000;
}

macro_rules! impl_has_checks {
	($($name:ident => $value:expr),*) => {
		$(
			fn $name(&self) -> bool {
				self.dbg_lv & $value == $value
			}
		)*
	}
}

#[derive(Debug)]
pub struct TokenizerParameters {
	multi_threaded: Option<usize>,
	pub dbg_lv: u8,
	pub bin_dat: Option<Vec<u8>>,
	pub bytes_to_read: Option<u64>,
	pub trainer_chk_bytes: Option<usize>,
}

impl TokenizerParameters {
	#![allow(dead_code)]
	impl_has_checks! {
		has_silent => debug_enum::SILENT,
		has_error => debug_enum::ERROR,
		has_warn => debug_enum::WARN,
		has_debug => debug_enum::DEBUG,
		has_info => debug_enum::INFO,
		has_verbose => debug_enum::VERBOSE,
		has_lengthy => debug_enum::LENGTHY
	}
}

fn vaildate_parameters(args: &Vec<String>) {
	let partial_arg_msg = (
		// Try not to go over 80 characters!
		format!("Usage: {} [parameter_1,parameter_2..] file\n", args[0])
		+ "E.g.: tokenizer_trainer_bin v=0b0_1111,br=0x3fff,mt=0 pexels-pixabay-302743.jpg\n"
		+ "Parameters are separated by commas, non-matches are ignored:\n"
		+ "  v=        Debug level; supports underscores, binary, decimal,\n"
		+ "            hexadecimal, and combined levels.\n"
		+ "            E.g., v=0b11 = Include error and warn.\n"
		+ "            v=0 = Include silent.\n"
		+ "            v=0b1 = Include error.\n"
		+ "            v=0b10 = Include warn.\n"
		+ "            v=0b100 = Include debug.\n"
		+ "            v=0b1000 = Include info.\n"
		+ "            v=0b1_0000 = Include verbose.\n"
		+ "            v=0b10_0000 = Include lengthy.\n"
		+ "  br=       Maximum bytes to read from the file.\n"
		+ "  mt=       Threads to use, 0 to detect system cores count.\n"
		+ "            None to use single.\n"
		+ "  tcb=      Training chunk bytes. Around 0(n ** 2 * 256) memory.\n"
	);

	if args.len() >= 4 {
		eprintln!("{}Hint: Please give 3 parameters; received {} parameters: {:?}", partial_arg_msg, args.len(), args);
		std::process::exit(1);
	} else if args.len() == 2 {
		eprintln!("{}Hint: Please specify the input file path.", partial_arg_msg);
		std::process::exit(1);
	} else if args.len() == 1 {
		eprintln!("{}Hint: Please specify options.", partial_arg_msg);
		std::process::exit(1);
	}
}

fn read_file(file_path: &String, bytes_to_read: Option<u64>) -> Option<Vec<u8>> {
	let bytes_to_read = bytes_to_read.unwrap_or(u64::MAX);

	let bin_file = match File::open(file_path) {
		Ok(file) => Some(file),
		Err(error) => {
			eprintln!("Unable to open {}: {}", file_path, error);
			let dir_path = Path::new(file_path).parent().unwrap();
			if dir_path.exists() || dir_path.to_str().unwrap() == "" {
				eprintln!("Hint: Found the directory; check the file name: {:?}", file_path);
			} else {
				eprintln!("Hint: Check the directory {:?}", dir_path);
			}
			std::process::exit(1);
		}
	}.unwrap();

	let reader = BufReader::new(bin_file);
	let mut data = vec![];
	reader.take(bytes_to_read).read_to_end(&mut data).expect("Unable to read file");
	Some(data)
}

trait ParseUInt: Sized {
	type Err;
	fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>;
}

macro_rules! impl_parse_uint {
	($t:ty) => {
		impl ParseUInt for $t {
			type Err = std::num::ParseIntError;
			fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
				<$t>::from_str_radix(s, radix)
			}
		}
	};
}

impl_parse_uint!(u8);
impl_parse_uint!(u16);
impl_parse_uint!(u32);
impl_parse_uint!(u64);
impl_parse_uint!(usize);

fn parse_uint<T: ParseUInt + std::str::FromStr>(options: &Vec<&str>, starts_with: &str) -> Option<T> {
	if let Some(br_str) = options.iter().find(|&s| s.starts_with(starts_with)) {
		if let Some(br_value) = br_str.split("=").nth(1).and_then(|string| {
			let s = string.replace("_", "");
			if s.starts_with("0x") {
				T::from_str_radix(&s[2..], 16).ok()
			} else if s.starts_with("0b") {
				T::from_str_radix(&s[2..], 2).ok()
			} else {
				s.parse::<T>().ok()
			}
		}) {
			return Some(br_value);
		} else {
			eprintln!("Error: Found \"{}\" but the number is invalid", starts_with);
			std::process::exit(1);
		}
	}
	None
}

fn process_cmd() {
	let mut tok_parameters = TokenizerParameters {
		dbg_lv: 0,
		bin_dat: None,
		bytes_to_read: None,
		trainer_chk_bytes: Some(16),
		multi_threaded: None,
	};
	let parameters: Vec<String> = env::args().collect();
	vaildate_parameters(&parameters);

	let options = parameters[1].split(",").collect::<Vec<&str>>();
	tok_parameters.bin_dat = read_file(&parameters[2], tok_parameters.bytes_to_read);
	tok_parameters.dbg_lv = parse_uint(&options, "v=").unwrap_or(debug_enum::SILENT);
	tok_parameters.bytes_to_read = parse_uint(&options, "br=");
	tok_parameters.multi_threaded = parse_uint(&options, "mt=");
	tok_parameters.trainer_chk_bytes = min(
		parse_uint(&options, "tcb="),
		Some(tok_parameters.bin_dat.clone().unwrap().len())
	);

	if tok_parameters.has_verbose() { println!("Verbose: args: {:?}", parameters); }
	if tok_parameters.has_verbose() { println!("Verbose: options: {:?}", options); }

	tok_trainer::entry(&mut tok_parameters);
	// tok_codec::demo();
}

fn main() {
	process_cmd();
}
