mod tok_trainer;
mod tok_codec;
use std::{env, fs::File, path::Path};


mod debug_enum {
#![allow(dead_code)]
	pub const SILENT: u8 = 0b_1;
	pub const HIGH_RISK: u8 = 0b10;
	pub const DEBUG: u8 = 0b100;
	pub const VERBOSE: u8 = 0b1000;
	pub const LENGTHY: u8 = 0b1_0000;
}

struct DebugStruct {
	lv: u8,
}

macro_rules! impl_has_checks {
	($($name:ident => $value:expr),*) => {
		$(
			fn $name(&self) -> bool {
				self.lv & $value == $value
			}
		)*
	}
}

impl DebugStruct {
	#![allow(dead_code)]
	impl_has_checks! {
		has_silent => debug_enum::SILENT,
		has_high_risk => debug_enum::HIGH_RISK,
		has_debug => debug_enum::DEBUG,
		has_verbose => debug_enum::VERBOSE,
		has_lengthy => debug_enum::LENGTHY
	}
}

const DBG_LV: DebugStruct = DebugStruct{ lv: 0b0_1111 };

pub struct TokenizerParameters {
	pub bin_file: Option<File>,
	pub bytes_to_read: Option<u64>,
	pub trainer_chk_len: Option<usize>,
}


fn vaildate_parameters(args: &Vec<String>) {
	if DBG_LV.has_verbose() { println!("args: {:?}", args); }
	let partial_arg_err_msg = format!("Usage: {} [parameter_1,parameter_2..] file\n", args[0]);

	if args.len() >= 4 {
		eprintln!("{}Hint: Please give 3 parameters. I received {} parameters: {:?}", partial_arg_err_msg, args.len(), args);
		return;
	} else if args.len() == 2 {
		eprintln!("{}Hint: Please specify the input file path.", partial_arg_err_msg);
		return;
	} else if args.len() == 1 {
		eprintln!("{}Hint: Please specify options.", partial_arg_err_msg);
		return;
	}
}

fn read_file(file_path: &String) -> Option<File> {
	match File::open(file_path) {
		Ok(file) => Some(file),
		Err(error) => {
			eprintln!("Unable to open {}: {}", file_path, error);
			let dir_path = Path::new(file_path).parent().unwrap();
			if dir_path.exists() || dir_path.to_str().unwrap() == "" {
				eprintln!("Hint: Check the file name: {:?}", file_path);
			} else {
				eprintln!("Hint: Check the directory {:?}", dir_path);
			}
			std::process::exit(1);
		}
	}
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
			eprintln!("Fatal error: found \"{}\" but the number is invalid", starts_with);
			std::process::exit(1);
		}
	}
	None
}

fn process_cmd() {
	let mut tok_parameters = TokenizerParameters {
		bin_file: None,
		bytes_to_read: None,
		trainer_chk_len: None,
	};

	let parameters: Vec<String> = env::args().collect();
	vaildate_parameters(&parameters);


	let options = parameters[1].split(",").collect::<Vec<&str>>();
	if DBG_LV.has_verbose() { println!("options: {:?}", options); }
	tok_parameters.bin_file = read_file(&parameters[2]);
	tok_parameters.bytes_to_read = parse_uint(&options, "br=");
	tok_parameters.trainer_chk_len = parse_uint(&options, "tcl=");

	tok_trainer::entry(&mut tok_parameters);
	// tok_codec::demo();
}

fn main() {
	process_cmd();
}
