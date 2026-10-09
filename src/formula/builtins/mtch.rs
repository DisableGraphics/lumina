use std::error::Error;

use crate::{Cell, container::Container, formula::{FormulaError::{LengthError, RuntimeError, TypeError}, builtins::val_eq, exec::{FormulaAstInner, FormulaExecutor}}};

fn vec_sub(a: &Vec<usize>, b: &Vec<usize>) -> Vec<usize> {
	a.iter().zip(b).map(|(a, b)| *a - *b).collect()
}

/// MATCH - Searches for a value in a range and returns its relative position.
///
/// Searches the range (second argument) for the search value (first argument) using exact match.
/// Returns a single-element range containing the position relative to the range's minimum coordinates.
///
/// # Arguments
/// * `args[0]` - Search value: `Number` or `Str` (not a Range)
/// * `args[1]` - Lookup range: A `Range` to search in
///
/// # Returns
/// * `Range` - Single-element range containing the relative position as a position vector
///   (e.g., for a 2D range starting at (0,0), position (0,1) means row 0, column 1)
///
/// # Errors
/// * `LengthError` - If not exactly 2 arguments provided
/// * `TypeError` - If first argument is a range, or second argument is not a range
/// * `RuntimeError` - If range is empty, or value not found in range
///
/// # Example
/// ```text
/// =MATCH("Apple", {0,0}:{0,4})  // Returns range with position of "Apple" in row
/// =MATCH(42, {0,0}:{2,2})  // Returns position of 42 in 3x3 grid
/// =MATCH("NotFound", {0,0}:{0,2})  // Returns error (not found)
/// // If range A1:C1 has "A","B","C", MATCH("B", A1:C1) returns position [0,1]
/// // If range A1:B2 has 1,2,3,4, MATCH(3, A1:B2) returns position [1,0]
/// ```
pub fn mtch(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("MATCH only accepts 2 arguments".to_string())));
	}

	let [argument, r] = args.as_slice() else {
		unreachable!()
	};

	if let FormulaAstInner::Range(_) = argument {
		return Err(Box::new(TypeError("First argument to MATCH must not be a range".to_string())));
	}

	let FormulaAstInner::Range(r) = r else {
		return Err(Box::new(TypeError("Second argument to MATCH must be a range".to_string())));
	};

	let Some(minimum) = r.iter().min_by(|a, b| a.1.cmp(&b.1)) else {
		return Err(Box::new(RuntimeError("MATCH did not find a minimum position".to_string(),)));
	};

	let (_, pos) = {
		let mut found = None;
		for (cell, pos) in r.iter() {
			let value = match &cell.content {
				crate::Content::Formula(formula) => exec._eval(formula, container)?,
				crate::Content::Number(value) => FormulaAstInner::Number(*value),
				crate::Content::Str(str) => FormulaAstInner::Str(str.clone()),
			};
			if val_eq(&value, argument, &super::CmpOperator::Eq) {
				found = Some((cell, pos));
				break;
			}
		}
		found.ok_or_else(|| Box::new(TypeError("Value not found in range".to_string())))?
	};

	Ok(FormulaAstInner::Range(vec![(Cell::default(), vec_sub(pos, &minimum.1))]))
}

#[cfg(test)]
mod test {

use crate::{Cell, Content, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=MATCH()").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn correct_len() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();

		let cell11 = Cell {
			content: Content::Str("Hello world".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,0], cell11).unwrap();

		let cell12 = Cell {
			content: Content::Str("Sandwich".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,1], cell12).unwrap();

		let cell13 = Cell {
			content: Content::Str("Potato".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,0,2], cell13).unwrap();

		let cell21 = Cell {
			content: Content::Number(1.0),
			..Default::default()
		};
		m.insert(&vec![0,1,0], cell21).unwrap();

		let cell22 = Cell {
			content: Content::Number(2.0),
			..Default::default()
		};
		m.insert(&vec![0,1,1], cell22).unwrap();

		let cell23 = Cell {
			content: Content::Number(5.0),
			..Default::default()
		};
		m.insert(&vec![0,1,2], cell23).unwrap();

		let ast = q.parse("=MATCH(\"Potato\", {0,0,1}:{0,1,2})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Range(n) => {
				assert_eq!(n.len(), 1);
				let pos = &n[0].1;
				assert_eq!(pos, &[0, 0, 1]);
			},
			_ => assert!(false)
		}
	}
}