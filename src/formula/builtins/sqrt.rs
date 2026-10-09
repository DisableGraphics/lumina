use std::error::Error;

use crate::{Cell, Content::Number, container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// SQRT - Returns the nth root of each numeric cell in a range.
///
/// This function returns a **Range** (not a single number), where each cell
/// in the input range is replaced with its nth root.
///
/// # Arguments
/// * `args[0]` - A `Range` containing cells to compute roots of
/// * `args[1]` - A `Number` specifying the root (default 2 = square root)
///
/// # Returns
/// * `Range` - A new range where each cell contains the nth root value
///
/// # Errors
/// * `LengthError` - If not exactly 2 arguments provided
/// * `TypeError` - If first argument is not a range, second argument is not a number,
///   or if any cell contains a string, or if a formula in the range evaluates to a range or string
///
/// # Example
/// ```text
/// =SQRT({0,0}:{2,2}, 2)  // Returns range with square roots
/// =SQRT({0,0}:{2,2}, 3)  // Returns range with cube roots
/// ```
pub fn sqrt(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("SQRT only accepts 2 arguments".to_string())));
	}

	let [range, pow] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("SQRT requires a range as its first argument".to_string())));
	};

	let FormulaAstInner::Number(pow) = pow else {
		return Err(Box::new(TypeError("SQRT requires a number as its second argument (root)".to_string())));
	};

	let mut ret = Vec::with_capacity(range.len());

	for (cell, p) in range {
		let value = match &cell.content {
			crate::Content::Formula(formula) => exec._eval(formula, container)?,
			crate::Content::Number(value) => FormulaAstInner::Number(*value),
			crate::Content::Str(_) => return Err(Box::new(TypeError("SQRT cannot compute roots of strings".to_string()))),
		};

		if matches!(value, FormulaAstInner::Range(_)) {
			return Err(Box::new(TypeError("SQRT range contains a formula that evaluates to a range".to_string())));
		}

		if matches!(value, FormulaAstInner::Str(_)) {
			return Err(Box::new(TypeError("SQRT range contains a formula that evaluates to a string".to_string())));
		}

		let FormulaAstInner::Number(v) = value else {
			unreachable!()
		};

		ret.push((Cell{content: Number(v.powf(1.0 / *pow)), ..Default::default()}, p.clone()));
	}

	Ok(FormulaAstInner::Range(ret))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::FormulaExecutor, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SQRT({0,0,0}:{1,1,1}, 2)").unwrap();

		let r = FormulaExecutor::new();
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		for i in 2..=3 {
			for j in 2..=3 {
				for k in 2..=3 {
					let cell = m.get_cell_at(&vec![i,j,k]).unwrap().unwrap();
					match cell.content {
						crate::Content::Number(n) => assert_eq!(n, 0.0),
						_ => assert!(false)
					}
				}
			}
		}
	}

	#[test]
	fn one_instance() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SQRT({0,0,0}:{1,1,1}, 2)").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Number(4.0);

		m.insert(&vec![0,0,1], def).unwrap();

		let r = FormulaExecutor::new();
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		for i in 2..=3 {
			for j in 2..=3 {
				for k in 2..=3 {
					let cell = m.get_cell_at(&vec![i,j,k]).unwrap().unwrap();
					match cell.content {
						crate::Content::Number(n) => assert!(n == 0.0 || n == 2.0),
						_ => assert!(false)
					}
				}
			}
		}
	}

	#[test]
	fn all_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SQRT({0,0,0}:{1,1,1}, 2)").unwrap();

		for i in 0..=1 {
			for j in 0..=1 {
				for k in 0..=1 {
					let mut def = Cell::default();
					def.content = crate::Content::Number(4.0);
					m.insert(&vec![i, j, k], def).unwrap();
				}
			}
		}

		let r = FormulaExecutor::new();
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		for i in 2..=3 {
			for j in 2..=3 {
				for k in 2..=3 {
					let cell = m.get_cell_at(&vec![i,j,k]).unwrap().unwrap();
					match cell.content {
						crate::Content::Number(n) => assert_eq!(n, 2.0),
						_ => assert!(false)
					}
				}
			}
		}
	}

	#[test]
	fn fails_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SQRT(\"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_num() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SQRT(1)").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}
}