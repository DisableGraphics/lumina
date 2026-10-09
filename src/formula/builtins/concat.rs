use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// CONCAT - Concatenates all string values in a range into a single string.
///
/// This function iterates over a range of cells, converts each cell's content to a string,
/// and concatenates them together. Empty cells are skipped. Numbers are converted to strings.
///
/// # Arguments
/// * `args[0]` - A `Range` containing cells to concatenate
///
/// # Returns
/// * `Str` - The concatenated string of all non-empty cells in the range
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If first argument is not a range, or if a formula in the range evaluates to a range
///
/// # Example
/// ```text
/// =CONCAT({0,0}:{2,2})  // Concatenates all cells in 3x3 range
/// // If A1="Hello", B1="World", C1=42: returns "HelloWorld42"
/// ```
pub fn concat(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("CONCAT only accepts 1 argument".to_string())));
	}

	let [range] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("CONCAT requires a range as its argument".to_string())));
	};

	let mut count = String::from("");

	for (cell, p) in range {
		if container.is_empty_cell(p) {
			continue;
		}
		let mut value = match &cell.content {
			crate::Content::Formula(formula) => exec._eval(formula, container)?,
			crate::Content::Number(value) => FormulaAstInner::Str(value.to_string()),
			crate::Content::Str(value) => FormulaAstInner::Str(value.to_owned()),
		};

		if matches!(value, FormulaAstInner::Range(_)) {
			return Err(Box::new(TypeError("CONCAT range contains a formula that evaluates to a range".to_string())));
		}

		if let FormulaAstInner::Number(c) = value {
			value = FormulaAstInner::Str(c.to_string())
		}

		count += match &value {
			FormulaAstInner::Str(f) => f,
			_ => unreachable!()
		};
	}

	Ok(FormulaAstInner::Str(count))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=CONCAT({0,0,0}:{1,1,1})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, ""),
			_ => assert!(false)
		}
	}

	#[test]
	fn one_instance() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=CONCAT({0,0,0}:{1,1,1})").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Str("Hello world".to_string());

		m.insert(&vec![0,0,1], def).unwrap();

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "Hello world"),
			_ => assert!(false)
		}
	}

	#[test]
	fn all_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=CONCAT({0,0,0}:{1,1,1})").unwrap();

		for i in 0..=1 {
			for j in 0..=1 {
				for k in 0..=1 {
					let mut def = Cell::default();
					def.content = crate::Content::Str("1".to_string());
					m.insert(&vec![i, j, k], def).unwrap();
				}
			}
		}

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Str(n) => assert_eq!(n, "11111111"),
			_ => assert!(false)
		}
	}

	#[test]
	fn fails_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=CONCAT(\"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_num() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=CONCAT(1)").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}
}