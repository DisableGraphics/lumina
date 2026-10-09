use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// TONUMBER - Parses a string or cell content as a number.
///
/// This function accepts a string, number, or single-cell range and attempts to
/// convert it to a numeric value. If the input is already a number, it's returned
/// as-is. If it's a string, it's parsed as a float. If it's a range, the cell's
/// content is evaluated (including formulas) and then converted.
///
/// # Arguments
/// * `args[0]` - A `Str`, `Number`, or single-cell `Range` to convert to a number
///
/// # Returns
/// * `Number` - The parsed numeric value
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided, or if a range contains more than one cell
/// * `TypeError` - If a range cell contains a formula that evaluates to a range,
///   or if a string cannot be parsed as a number
///
/// # Example
/// ```text
/// =TONUMBER("123")  // Returns 123.0
/// =TONUMBER(42)  // Returns 42.0
/// =TONUMBER({0,0})  // Returns numeric value of cell at [0,0]
/// =TONUMBER("3.14")  // Returns 3.14
/// ```
pub fn tonumber(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("TONUMBER only accepts 1 argument".to_string())));
	}

	let [msg] = args.as_slice() else {
		unreachable!()
	};

	let value = match msg {
		FormulaAstInner::Range(items) => {
			if items.len() != 1 {
				return Err(Box::new(LengthError("TONUMBER only accepts a range with 1 cell".to_string())));
			}
			match &items[0].0.content {
				crate::Content::Formula(formula_ast) => exec._eval(&formula_ast, container)?,
				crate::Content::Number(i) => FormulaAstInner::Number(*i),
				crate::Content::Str(str) => FormulaAstInner::Str(str.to_string()),
			}
		},
		FormulaAstInner::Number(num) => FormulaAstInner::Number(*num),
		FormulaAstInner::Str(str) => FormulaAstInner::Str(str.to_string())
	};

	match value {
		FormulaAstInner::Range(_) => {
			return Err(Box::new(TypeError("TONUMBER range contains a formula that evaluates to a range".to_string())));
		},
		FormulaAstInner::Number(i) => Ok(FormulaAstInner::Number(i)),
		FormulaAstInner::Str(str) => Ok({
			let i = str.parse()?;
			FormulaAstInner::Number(i)
		}),
	}
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TONUMBER()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TONUMBER({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(i) => assert_eq!(i, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_on_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TONUMBER(\"12\")").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell::default();
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(i) => assert_eq!(i, 12.0),
			_ => assert!(false)
		}
	}
}