use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}, parser::FormulaParser}};

/// IFERROR - Evaluates an expression and returns a fallback value if it errors.
///
/// The first argument is a string expression that gets parsed and evaluated (like a formula).
/// If the evaluation succeeds, its result is returned. If it produces any error, the second
/// argument is returned instead.
///
/// # Arguments
/// * `args[0]` - A `Str` containing a parseable expression (e.g., "TONUMBER({0,0})", "1/0")
/// * `args[1]` - The value to return if the first argument errors (any type: Number, Str, Range)
///
/// # Returns
/// * The result of the first argument if it evaluates successfully, otherwise the second argument
///
/// # Errors
/// * `LengthError` - If not exactly 2 arguments provided
/// * `TypeError` - If first argument is not a string, or if the expression cannot be parsed
///
/// # Example
/// ```text
/// =IFERROR("TONUMBER({0,0}) / TONUMBER({0,1})", 0)  // Returns 0 if division by zero
/// =IFERROR("XLOOKUP(\"notfound\", {0,0}:{0,9}, {1,0}:{1,9})", "N/A")  // Returns "N/A" if not found
/// =IFERROR("1/0", "Error")  // Returns "Error"
/// =IFERROR("TONUMBER({0,0})", 42)  // Returns cell value if numeric, 42 otherwise
/// ```
pub fn iferror(mut args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("IFERROR only accepts 2 arguments".to_string())));
	}

	let [cell, _] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Str(cell) = cell else {
		return Err(Box::new(TypeError("First argument to IFERROR must be a string with a parseable expression inside".to_string())));
	};

	let parser = FormulaParser::new();
	let ast = parser.parse(format!("={cell}"))?;

	let exprval = exec._eval(&ast, container);
	let onerror = args.remove(1);
	Ok(match exprval {
		Ok(ok) => ok,
		Err(_e) => onerror
	})
}

#[cfg(test)]
mod test {
	use crate::{Cell, Content::Str, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IFERROR()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IFERROR(\"TONUMBER({0,0,0}:{0,0,1})\", 7)").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=IFERROR(\"TOSTRING({0,0,0})\", 7)").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell {
			content: Str("Potato".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "Potato"),
			_ => assert!(false)
		}
	}
}