# celtic

Celtic is a small macro-compiler to allow for very basic meta-programming in languages that don't have the ability to do at compile time.
In Celtic there is different commands which are put in square brackets, which can either output text or be used in if expressions to conditionally show text.


## Basic Example 

```
#[env(TARGET)]
#[include(tests/foo.test)]

#[if: eq(DEVELOPMENT, env(TARGET))]
Rubberducky time boi
#[endif]

#[if: eq(RELEASE, env(TARGET))]
It's action time yo
#[endif]
```

## Implementing commands

Commands are just functions that operate on `MacroValue`'s ;)

```rust
pub fn include(params: Vec<MacroValue>) -> Result<MacroValue, Box<Error>> {
	assert!(params.len() == 1);
	
	let mut file = File::open(params[0].clone().try_as_string()?)?;
	let mut buffer = String::new();
	file.read_to_string(&mut buffer)?;
	
	return Ok(MacroValue::STRING(buffer));
}
```
