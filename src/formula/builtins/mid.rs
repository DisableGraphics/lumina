use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// Returns the current date and time as a UNIX timestamp
pub fn mid(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() < 1 || args.len() > 2 {
		return Err(Box::new(LengthError("MID only accepts 1 or 2 arguments".to_string())));
	}

	let text = match args.get(0).unwrap() {
		FormulaAstInner::Str(s) => s,
		_ => return Err(Box::new(TypeError("First argument to MID must be a string".to_string())))
	};

	let num = match args.get(1) {
		Some(FormulaAstInner::Number(n)) => *n,
		None => 1.0,
		_ => return Err(Box::new(TypeError("Second argument to MID must be a number or nothing".to_string())))
	};

	let chars: Vec<char> = text.chars().collect();
	let n = (num as usize).min(chars.len());

	let start = (chars.len() - n) / 2;
	let result: String = chars[start..start + n].iter().collect();

	Ok(FormulaAstInner::Str(result))

}

#[cfg(test)]
mod test {
	use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID(\"Hello world\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, " "),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_multiple() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID(\"Hello world\", 3)").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "o w"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_empty_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MID(\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, ""),
			_ => assert!(false)
		}
	}
}