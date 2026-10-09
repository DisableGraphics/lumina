use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::ifeval, exec::{FormulaAstInner, FormulaExecutor}}};

/// AND - Returns true (1) if all conditions evaluate to true, false (0) otherwise.
///
/// Each argument is a string expression that gets parsed and evaluated. All must
/// evaluate to true (non-zero) for the result to be true.
///
/// # Arguments
/// * `args[0..]` - Two or more `Str` values, each containing a parseable expression
///
/// # Returns
/// * `Number` - 1.0 if all conditions are true, 0.0 otherwise
///
/// # Errors
/// * `LengthError` - If fewer than 2 arguments provided
/// * `TypeError` - If any argument is not a string, or if any expression cannot be parsed/evaluated
///
/// # Example
/// ```text
/// =AND("TONUMBER({0,0}) > 5", "TONUMBER({0,1}) < 10")  // Returns 1 if both true
/// =AND("ISTEXT({0,0})", "ISNUMBER({0,1})")  // Returns 1 if A1 is text and B1 is number
/// =AND("1==1", "2==2", "3==3")  // Returns 1 (all true)
/// ```
pub fn and(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() < 2 {
		return Err(Box::new(LengthError("AND requires at least 2 arguments".to_string())));
	}

	let mut all_true = true;

	for arg in args {
		let FormulaAstInner::Str(cell) = arg else {
			return Err(Box::new(TypeError("AND requires string arguments (condition expressions)".to_string())));
		};

		let exprval = ifeval(&cell, container, exec)?;
		if !exprval {
			all_true = false;
			break;
		}
	}

	Ok(FormulaAstInner::Number(match all_true {
		true => 1.0,
		false => 0.0
	}))
}

#[cfg(test)]
mod test {
	use crate::{Cell, Content::Str, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AND()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AND(\"TONUMBER({0,0,0}) == 0\", \"TOSTRING({0,0,0}) == \\\"Sandwich\\\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AND(\"TONUMBER({0,0,1}) == 0\", \"TOSTRING({0,0,0}) == \\\"Potato\\\"\")").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: Str("Potato".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_three_args() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AND(\"1==1\", \"2==2\", \"3==3\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_false() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AND(\"1==1\", \"2==3\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}
}