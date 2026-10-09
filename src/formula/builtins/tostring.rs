use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// TOSTRING - Converts a value (number, string, or cell content) to a string.
///
/// This function accepts a number, string, or single-cell range and converts it
/// to a string representation. If the input is already a string, it's returned
/// as-is. If it's a number, it's converted to a string. If it's a range, the cell's
/// content is evaluated (including formulas) and then converted.
///
/// # Arguments
/// * `args[0]` - A `Number`, `Str`, or single-cell `Range` to convert to a string
///
/// # Returns
/// * `Str` - The string representation of the value
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided, or if a range contains more than one cell
/// * `TypeError` - If a range cell contains a formula that evaluates to a range
///
/// # Example
/// ```text
/// =TOSTRING(123)  // Returns "123"
/// =TOSTRING(3.14)  // Returns "3.14"
/// =TOSTRING("hello")  // Returns "hello"
/// =TOSTRING({0,0})  // Returns string value of cell at [0,0]
/// ```
pub fn tostring(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("TOSTRING only accepts 1 argument".to_string())));
	}

	let [msg] = args.as_slice() else {
		unreachable!()
	};

	let value = match msg {
		FormulaAstInner::Range(items) => {
			if items.len() != 1 {
				return Err(Box::new(LengthError("TOSTRING only accepts a range with 1 cell".to_string())));
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
			return Err(Box::new(TypeError("TOSTRING range contains a formula that evaluates to a range".to_string())));
		},
		FormulaAstInner::Number(i) => Ok(FormulaAstInner::Str(i.to_string())),
		FormulaAstInner::Str(str) => Ok(FormulaAstInner::Str(str)),
	}
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TOSTRING()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TOSTRING({0,0,0})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(i) => assert_eq!(i, "0"),
			_ => assert!(false)
		}
	}

	#[test]
	fn correct_on_number() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=TOSTRING(12)").unwrap();
		let r = FormulaExecutor::new();
		let cell = Cell::default();
		m.insert(&vec![0,0,0], cell).unwrap();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(i) => assert_eq!(i, "12"),
			_ => assert!(false)
		}
	}
}