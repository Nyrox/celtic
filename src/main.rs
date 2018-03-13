extern crate dotenv;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Peekable;

mod macros;

#[derive(Default, Debug)]
struct Options {
	input: Option<String>,
	out: Option<String>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MacroValue {
	BOOL(bool),
	STRING(String)
}

impl MacroValue {
	pub fn try_as_string(self) -> Result<String, &'static str>{
		match self {
			MacroValue::STRING(s) => Ok(s),
			_ => Err("")
		}
	}
}

impl From<MacroValue> for bool {
	fn from(value: MacroValue) -> bool {
		match value {
			MacroValue::BOOL(b) => b,
			MacroValue::STRING(ref s) => {
				s.to_lowercase() == "true"
			}			
		}
	}
}

struct ExecutionContext<'a> {
	part: &'a str
}

impl<'a> ExecutionContext<'a> {
	pub fn new(part: &'a str) -> ExecutionContext<'a> {
		ExecutionContext { part }
	}
	
	pub fn execute(&mut self) -> String {
		let mut output = String::new();
		
		let mut chars = self.part.chars().peekable();
		while let Some(c) = chars.next() {	
			if c == '#' {
				if chars.peek().unwrap() == &'[' {
					chars.next();
					// Heeey we encountered a macro
					let mut macro_inner = (|| {
						let mut buffer = String::new();
						while let Some(c) = chars.next() {
							if c != ']' { buffer.push(c); continue; }
							break;
						}
						return buffer;
					})();
					

					
					println!("{}", macro_inner);
					
					let mut depth = 1;
					if macro_inner.starts_with("if:") {
						let mut buf = String::new();
						
						'killme: while let Some(c) = chars.next() {
							if c == '#' {
								if chars.peek().unwrap() == &'[' {
									chars.next();
									let macro_inner = (|| { 
										let mut buffer = String::new();
										while let Some(c) = chars.next() {
											if c != ']' { buffer.push(c); continue; }
											break;
										}
										return buffer;
									})();
									
									if macro_inner.starts_with("endif") {
										depth -= 1;
										if depth == 0 { break 'killme; }
									}
									else {
										buf.push_str(&format!("#[{}]", macro_inner));
									}
									
									continue;
								}
								else {
									continue;
								}							
							}
							buf.push(c);
						}
						
						println!("<Buffer>{}</Buffer>", buf);
						
						macro_inner = macro_inner.split_off(3);
						let _macro = parse_macro_invocation(macro_inner);
						
						
						if bool::from(evaluate_macro(_macro)) {
							output.push_str(&ExecutionContext::new(&buf).execute());
						}
						
						
					}
					else {
						let _macro = parse_macro_invocation(macro_inner);						
						output.push_str(&evaluate_macro(_macro).try_as_string().unwrap());
					}
					
					continue;				
				}
			}			
			output.push(c);	
		}
	
		return output;
	}
}

fn main() {
	dotenv::dotenv().ok();
	
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
	
	
	let mut output = ExecutionContext::new(&buffer).execute();
	
	
	
	let mut outFile = File::create(options.out.unwrap()).unwrap();
	outFile.write_all(output.as_bytes());
}

fn evaluate_macro(_macro: (String, Vec<MacroValue>)) -> MacroValue {
	let (function, arguments) = _macro;
	
	match &*function {
		"include" => macros::include(arguments).unwrap(),
		"env" => MacroValue::STRING(env::var(arguments[0].clone().try_as_string().unwrap()).unwrap()),
		"eq" => macros::eq(arguments).unwrap(),
		_ => panic!("{} is not a vaild macro.", function)
	}
}

fn env(arguments: Vec<String>) -> String {
	return env::var(&arguments[0]).unwrap_or("undefined".to_string());
}

fn parse_macro_invocation(content: String) -> (String, Vec<MacroValue>) {
	let mut chars = content.chars().peekable();
	while chars.peek() == Some(&' ') { chars.next(); }
	
	let word = parse_word_alphanumeric(&mut chars);

	// TODO: Add keyword checks
	
	// Parse argument
	// Skip opening bracket
	chars.next();
	
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

fn parse_arguments<It: Iterator<Item=char>>(chars: &mut Peekable<It>) -> Vec<MacroValue> {
	let mut arguments = Vec::new();
	
	'parse: loop {
		while chars.peek() == Some(&' ') { chars.next(); }
		
		// arguments.push(parse_word_if(chars, |c| {
		// 	c != '(' && c != ')' && c != ',' 
		// }));
		
		let mut buffer = String::new();
		while let Some(c) = chars.next() {
			
			if c == '(' {
				arguments.push(evaluate_macro((buffer, parse_arguments(chars))));
				chars.next();
				continue 'parse;
			}
			
			if c == ',' {
				arguments.push(MacroValue::STRING(buffer));
				continue 'parse;
			}
			
			if c == ')' {
				arguments.push(MacroValue::STRING(buffer));
				break 'parse;
			}
			
			buffer.push(c);
		}
		
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