use super::MacroValue;

use std::fs::File;
use std::error::Error;
use std::io::prelude::*;

pub fn include(params: Vec<MacroValue>) -> Result<MacroValue, Box<Error>> {
	assert!(params.len() == 1);
	
	let mut file = File::open(params[0].clone().try_as_string()?)?;
	let mut buffer = String::new();
	file.read_to_string(&mut buffer)?;
	
	return Ok(MacroValue::STRING(buffer));
}

pub fn eq(params: Vec<MacroValue>) -> Result<MacroValue, Box<Error>> {
	Ok(MacroValue::BOOL(params[0] == params[1]))
}