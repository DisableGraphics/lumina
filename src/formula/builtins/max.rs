use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// MAX - Returns the maximum numeric value in a range.
///
/// # Arguments
/// * `args[0]` - A `Range` containing cells to evaluate
///
/// # Returns
/// * `Number` - The maximum of all numeric cells in the range
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If first argument is not a range, or if any cell contains a string,
///   or if a formula in the range evaluates to a range or string
///
/// # Example
/// ```text
/// =MAX({0,0}:{2,2})  // Returns maximum of all cells in 3x3 range starting at origin
/// ```
pub fn max(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("MAX only accepts 1 argument".to_string())));
	}

	let [range] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("MAX requires a range as its argument".to_string(),)));
	};

	let mut max_val = f64::NEG_INFINITY;
	let mut has_values = false;

	for (cell, _) in range {
		let value = match &cell.content {
			crate::Content::Formula(formula) => exec._eval(formula, container)?,
			crate::Content::Number(value) => FormulaAstInner::Number(*value),
			crate::Content::Str(_) => return Err(Box::new(TypeError("MAX cannot find maximum of strings".to_string()))),
		};

		if matches!(value, FormulaAstInner::Range(_)) {
			return Err(Box::new(TypeError("MAX range contains a formula that evaluates to a range".to_string())));
		}

		if matches!(value, FormulaAstInner::Str(_)) {
			return Err(Box::new(TypeError("MAX range contains a formula that evaluates to a string".to_string())));
		}

		if let FormulaAstInner::Number(f) = value {
			max_val = max_val.max(f);
			has_values = true;
		}
	}

	if !has_values {
		return Err(Box::new(TypeError("MAX range contains no numeric values".to_string())));
	}

	Ok(FormulaAstInner::Number(max_val))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MAX({0,0,0}:{1,1,1})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn one_instance() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MAX({0,0,0}:{1,1,1})").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Number(12.0);

		m.insert(&vec![0,0,1], def).unwrap();

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn all_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MAX({0,0,0}:{1,1,1})").unwrap();

		for i in 0..=1 {
			for j in 0..=1 {
				for k in 0..=1 {
					let mut def = Cell::default();
					def.content = crate::Content::Number(12.0);
					m.insert(&vec![i, j, k], def).unwrap();
				}
			}
		}

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn multiple_values() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MAX({0,0,0}:{1,1,1})").unwrap();

		let mut def1 = Cell::default();
		def1.content = crate::Content::Number(5.0);
		m.insert(&vec![0,0,0], def1).unwrap();

		let mut def2 = Cell::default();
		def2.content = crate::Content::Number(12.0);
		m.insert(&vec![0,0,1], def2).unwrap();

		let mut def3 = Cell::default();
		def3.content = crate::Content::Number(-3.0);
		m.insert(&vec![1,1,1], def3).unwrap();

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn fails_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MAX(\"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_num() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MAX(1)").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}
}