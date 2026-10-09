use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// Returns the current date and time as a UNIX timestamp
pub fn substitute(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 3 {
		return Err(Box::new(LengthError("SUBSTITUTE only accepts 3 arguments".to_string())));
	}

	let [str, needle, subs] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(str) = str else {
		return Err(Box::new(TypeError("First argument to SUBSTITUTE must be a string".to_string(),)));
	};

	let FormulaAstInner::Str(needle) = needle else {
		return Err(Box::new(TypeError("Second argument to SUBSTITUTE must be a string".to_string(),)));
	};

	let FormulaAstInner::Str(subs) = subs else {
		return Err(Box::new(TypeError("Third argument to SUBSTITUTE must be a string".to_string(),)));
	};
	
	Ok(FormulaAstInner::Str(str.replace(needle, subs)))
}

#[cfg(test)]
mod test {
	use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUBSTITUTE()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUBSTITUTE(\" Hello world    \", \" \", \"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "Helloworld"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_empty_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUBSTITUTE(\"\", \"\", \"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, ""),
			_ => assert!(false)
		}
	}
}