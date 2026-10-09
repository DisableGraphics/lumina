use std::error::Error;

use crate::{Cell, container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

/// SORT - Sorts the values in a range and returns a new sorted range.
///
/// This function takes a range of cells, evaluates each cell's content (including formulas),
/// converts all values to strings, sorts them (numbers numerically, strings lexicographically),
/// and returns a new range with the sorted values placed starting from the minimum position
/// of the original range.
///
/// # Arguments
/// * `args[0]` - A `Range` containing cells to sort
///
/// # Returns
/// * `Range` - A new range with the same dimensions, containing sorted values as strings
///
/// # Errors
/// * `LengthError` - If not exactly 1 argument provided
/// * `TypeError` - If first argument is not a range, or if a formula in the range evaluates to a range
///
/// # Example
/// ```text
/// =SORT({0,0}:{2,2})  // Sorts all cells in 3x3 range
/// // If range contains: 5, 2, 8, "apple", "banana"
/// // Returns range with: 2, 5, 8, "apple", "banana"
/// ```
pub fn sort(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 1 {
		return Err(Box::new(LengthError("SORT only accepts 1 argument".to_string())));
	}

	let [range] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("SORT requires a range as its argument".to_string())));
	};

	let mut sorteable = Vec::with_capacity(range.len());

	for (cell, p) in range {
		let value = match &cell.content {
			crate::Content::Formula(formula) => exec._eval(formula, container)?,
			crate::Content::Number(value) => FormulaAstInner::Str(value.to_string()),
			crate::Content::Str(v) => FormulaAstInner::Str(v.to_string()),
		};

		if matches!(value, FormulaAstInner::Range(_)) {
			return Err(Box::new(TypeError("SORT range contains a formula that evaluates to a range".to_string())));
		}

		let FormulaAstInner::Str(s) = value else {
			unreachable!()
		};

		sorteable.push((s, p.clone()));
	}
	sorteable.sort_by(|a, b| {
		match (a.0.parse::<f64>(), b.0.parse::<f64>()) {
			(Ok(a), Ok(b)) => a.total_cmp(&b),
			(Err(_), Err(_)) => a.0.cmp(&b.0),
			(Ok(_), Err(_)) => std::cmp::Ordering::Less,
			(Err(_), Ok(_)) => std::cmp::Ordering::Greater,
		}
	});


	let minimum = range
		.iter()
		.min_by(|a, b| a.1.cmp(&b.1))
		.unwrap()
		.1
		.clone();

	Ok(FormulaAstInner::Range(
		sorteable
			.into_iter()
			.enumerate()
			.map(|(index, (str, _))| {
				let pos = minimum.iter().zip(range[index].1.clone()).map(|x| {
					x.0 + x.1
				}).collect();
				(
					Cell {
						content: crate::Content::Str(str),
						..Default::default()
					},
					pos,
				)
			})
			.collect()
	))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::FormulaExecutor, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SORT({0,0,0}:{1,1,1})").unwrap();

		let r = FormulaExecutor::new();
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		for i in 2..=3 {
			for j in 2..=3 {
				for k in 2..=3 {
					let cell = m.get_cell_at(&vec![i,j,k]).unwrap().unwrap();
					match cell.content {
						crate::Content::Str(n) => assert_eq!(n, "0"),
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
		let ast = q.parse("=SORT({0,0,0}:{1,1,1})").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Number(-12.0);

		m.insert(&vec![0,0,1], def).unwrap();

		let r = FormulaExecutor::new();
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		assert_eq!(m.get_cell_at(&vec![2,2,2]).unwrap().unwrap().content, crate::Content::Str("-12".to_string()));
		for i in 2..=3 {
			for j in 2..=3 {
				for k in 2..=3 {
					if !(i == j && j == k && i == 2) {
						let cell = m.get_cell_at(&vec![i,j,k]).unwrap().unwrap();
						match cell.content {
							crate::Content::Str(n) => assert_eq!(n, "0"),
							_ => assert!(false)
						}
					}
				}
			}
		}
	}

	#[test]
	fn all_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SORT({0,0,0}:{1,1,1})").unwrap();

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
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		for i in 2..=3 {
			for j in 2..=3 {
				for k in 2..=3 {
					let cell = m.get_cell_at(&vec![i,j,k]).unwrap().unwrap();
					match cell.content {
						crate::Content::Str(n) => assert_eq!(n, "12"),
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
		let ast = q.parse("=SORT(\"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_num() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=SORT(1)").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}
}