use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::{CmpOperator, get_operator, val_eq}, exec::{FormulaAstInner, FormulaExecutor}}};

/// SUMIF - Sums cells in a range matching a criteria.
///
/// Iterates through all cells in the range and sums numeric values of those matching the criteria.
/// The criteria can be:
/// - A number: exact match (e.g., `12`)
/// - An expression: comparison operator + value (e.g., `">= 2"`, `"< 10"`, `"<> 5"`)
///
/// String criteria are not supported for SUMIF (use COUNTIF for string matching).
/// Cells containing text or errors are skipped (not summed, not counted as matches).
///
/// Supported operators: `==`, `!=`, `>`, `<`, `>=`, `<=`
///
/// # Arguments
/// * `args[0]` - A `Range` of cells to evaluate and sum
/// * `args[1]` - A `Number` or `Str` with comparison expression (not a Range)
///
/// # Returns
/// * `Number` - The sum of matching numeric cells (as f64)
///
/// # Errors
/// * `LengthError` - If not exactly 2 arguments provided
/// * `TypeError` - If first argument is not a range, second is a range, criteria is a string (not expression), or range contains strings/ranges
///
/// # Example
/// ```text
/// =SUMIF({0,0}:{2,2}, 5)  // Sums cells equal to 5
/// =SUMIF({0,0}:{2,2}, ">= 10")  // Sums cells >= 10
/// =SUMIF({0,0}:{2,2}, "<> 0")  // Sums non-zero cells
/// =SUMIF({0,0}:{2,2}, "< 0")  // Sums negative cells
/// // If A1=5, B1=10, C1="text", D1=3: SUMIF(range, ">=5") = 15
/// ```
pub fn sumif(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("SUMIF only accepts 2 arguments".to_string())));
	}

	let [range, criteria] = args.as_slice() else {
		unreachable!()
	};

	if matches!(criteria, FormulaAstInner::Range(_)) {
		return Err(Box::new(TypeError("Second argument to SUMIF must not be a range".to_string(),)));
	}

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("First argument to SUMIF must be a range".to_string(),)));
	};

	let (op, criteria) = match criteria {
		FormulaAstInner::Range(_) => unreachable!(),
		FormulaAstInner::Number(_) => (CmpOperator::Eq, criteria.clone()),
		FormulaAstInner::Str(s) => get_operator(s)
	};

	if matches!(criteria, FormulaAstInner::Str(_)) {
		return Err(Box::new(TypeError("SUMIF cannot sum strings".to_string())))
	}

	let mut count = 0.0;

	for (cell, _) in range {
		let value = match &cell.content {
			crate::Content::Formula(formula) => exec._eval(formula, container)?,
			crate::Content::Number(value) => FormulaAstInner::Number(*value),
			crate::Content::Str(_) => return Err(Box::new(TypeError("SUMIF cannot sum strings".to_string()))),
		};

		if matches!(value, FormulaAstInner::Range(_)) {
			return Err(Box::new(TypeError("SUMIF range contains a formula that evaluates to a range".to_string())));
		}

		if matches!(value, FormulaAstInner::Str(_)) {
			return Err(Box::new(TypeError("SUMIF range contains a formula that evaluates to a string".to_string())));
		}

		if val_eq(&value, &criteria, &op) {
			count += match value {
				FormulaAstInner::Number(f) => f,
				_ => unreachable!()
			};
		}
	}

	Ok(FormulaAstInner::Number(count))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUMIF({0,0,0}:{1,1,1}, 12)").unwrap();
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
		let ast = q.parse("=SUMIF({0,0,0}:{1,1,1}, 12)").unwrap();

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
		let ast = q.parse("=SUMIF({0,0,0}:{1,1,1}, 12)").unwrap();

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
			FormulaAstInner::Number(n) => assert_eq!(n, 8.0*12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn fails_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUMIF({0,0,0}:{1,1,1}, \"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_range() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUMIF({0,0,0}:{1,1,1}, {0,0,0}:{0,0,1})").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn all_instances_expression() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUMIF({0,0,0}:{1,1,1}, \">= 2\")").unwrap();

		for i in 0..=1 {
			for j in 0..=1 {
				for k in 0..=1 {
					let mut def = Cell::default();
					def.content = crate::Content::Number((i+j+k) as f64);
					m.insert(&vec![i, j, k], def).unwrap();
				}
			}
		}

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 9.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn all_instances_expression2() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SUMIF({0,0,0}:{1,1,1}, \"> 2\")").unwrap();

		for i in 0..=1 {
			for j in 0..=1 {
				for k in 0..=1 {
					let mut def = Cell::default();
					def.content = crate::Content::Number((i+j+k) as f64);
					m.insert(&vec![i, j, k], def).unwrap();
				}
			}
		}

		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 3.0),
			_ => assert!(false)
		}
	}

}