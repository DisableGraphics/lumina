use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::ifeval, exec::{FormulaAstInner, FormulaExecutor}}};

/// NOT - Returns true (1) if the condition evaluates to false, false (0) otherwise.
///
/// The argument is a string expression that gets parsed and evaluated. The result
/// is logically inverted.
///
/// # Arguments
/// * `args[0]` - A `Str` containing a parseable expression
///
/// # Returns
/// * `Number` - 1.0 if condition is false, 0.0 if condition is true
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If argument is not a string, or if the expression cannot be parsed/evaluated
///
/// # Example
/// ```text
/// =NOT("TONUMBER({0,0}) > 5")  // Returns 1 if A1 <= 5, 0 if A1 > 5
/// =NOT("ISTEXT({0,0})")  // Returns 1 if A1 is not text
/// =NOT("1==1")  // Returns 0 (condition is true)
/// =NOT("1==2")  // Returns 1 (condition is false)
/// ```
pub fn not(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("NOT only accepts 1 argument".to_string())));
	}

	let [cell1] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(cell1) = cell1 else {
		return Err(Box::new(TypeError("NOT requires a string argument (condition expression)".to_string())));
	};

	let exprval1 = ifeval(&cell1, container, exec)?;

	Ok(FormulaAstInner::Number(match exprval1 {
		true => 0.0,
		false => 1.0
	}))
}

#[cfg(test)]
mod test {
	use crate::{Cell, Content::Str, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=NOT()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=NOT(\"TOSTRING({0,0,0}) == \\\"Sandwich\\\"\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=NOT(\"TONUMBER({0,0,1}) == 0\")").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: Str("Potato".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_false() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=NOT(\"1==2\")").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}
}