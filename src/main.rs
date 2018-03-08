use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Peekable;

#[derive(Default, Debug)]
struct Options {
	input: Option<String>,
	out: Option<String>
}

fn main() {
	let mut options = Options::default();
	
	let mut it_arg = env::args().skip(1);
	
	while let Some(arg) = it_arg.next() {
		if arg == "-o" {
			options.out = Some(it_arg.next().unwrap());
			continue;
		}
		
		// If it's not a flag it must be the input file
		if let Some(input) = options.input {
			panic!("Can't have multiple input parameters. Aborting.");
		}
		
		options.input = Some(arg);
	}
	
	if options.input.is_none() {
		panic!("Need to supply atleast one input parameter. Aborting.");
	}
	
	println!("{:?}", options);
	
	
	let mut input_file = File::open(options.input.clone().unwrap()).unwrap();
	let mut buffer = String::new();
	input_file.read_to_string(&mut buffer);
	
	println!("{}", buffer);
	
	let mut output = String::new();
	
	// Parse the input File
	let mut chars = buffer.chars().peekable();
	while let Some(c) = chars.next() {	
		if c == '#' {
			if chars.peek().unwrap() == &'[' {
				chars.next();
				// Heeey we encountered a macro
				let macro_inner = (|| {
					let mut buffer = String::new();
					while let Some(c) = chars.next() {
						if c != ']' { buffer.push(c); continue; }
						break;
					}
					return buffer;
				})();
				
				let _macro = parse_macro_invocation(macro_inner);
				
				output.push_str(&evaluate_macro(_macro));
				
				continue;				
			}
		}
		
		output.push(c);
	}
	
	let mut outFile = File::create(options.out.unwrap()).unwrap();
	outFile.write_all(output.as_bytes());
	println!("{}", output);
}

fn evaluate_macro(_macro: (String, Vec<String>)) -> String {
	let (function, arguments) = _macro;
	
	// TODO: Implement proper plugins
	
	match &*function {
		"include" => include(arguments),
		_ => panic!("{} is not a vaild macro.", function)
	}
}

fn include(arguments: Vec<String>) -> String {
	println!("{:?}", arguments);
	let mut f = File::open(&arguments[0]).unwrap();
	let mut s = String::new();
	f.read_to_string(&mut s);
	
	return s;
}

fn parse_macro_invocation(content: String) -> (String, Vec<String>) {
	let mut chars = content.chars().peekable();
	while chars.peek() == Some(&' ') { chars.next(); }
	
	let word = parse_word_alphanumeric(&mut chars);

	// TODO: Add keyword checks
	
	// Parse argument
	// Skip opening bracket
	println!("{:?}", chars.next());
	
	let mut arguments = Vec::new();
	if chars.peek().unwrap().is_alphanumeric() {
		arguments = parse_arguments(&mut chars);
	}
	
	// TODO: Support macro invocations as arguments?
	
	// Skip closing bracket
	chars.next();
	
	println!("{}, {:?}", word, arguments);
	(word, arguments)
}

fn parse_arguments<It: Iterator<Item=char>>(chars: &mut Peekable<It>) -> Vec<String> {
	let mut arguments = Vec::new();
	
	'parse: loop {
		while chars.peek() == Some(&' ') { chars.next(); }
		
		arguments.push(parse_word_if(chars, |c| {
			c != ')' && c != ',' 
		}));
		
		while chars.peek() == Some(&' ') { chars.next(); }
		if chars.peek() != Some(&',') { break 'parse; }
	}
	
	return arguments;
}

/*
Reads a string until it hit's a character that isn't alpha numeric
*/
fn parse_word_alphanumeric<It: Iterator<Item=char>>(chars: &mut Peekable<It>) -> String {
	let mut buffer = String::new();
	
	while chars.peek().unwrap().is_alphanumeric() {
		buffer.push(chars.next().unwrap());
	}
	
	return buffer;
}

fn parse_word_if<It: Iterator<Item=char>, Cond: Fn(char) -> bool>(chars: &mut It, cond: Cond) -> String {
	let mut buffer = String::new();
	
	while let Some(c) = chars.next() {
		if !cond(c) { break; }
		buffer.push(c);
	}
	
	return buffer;
}