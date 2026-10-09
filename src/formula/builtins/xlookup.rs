use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::val_eq, exec::{FormulaAstInner, FormulaExecutor}}};

/// XLOOKUP - Searches for a value in one range and returns the corresponding value from another range.
///
/// Searches the lookup range (second argument) for the search value (first argument) using exact match.
/// When found, returns the value at the same position from the return range (third argument).
/// If not found, returns an empty string.
///
/// Both ranges must have the same length. The search uses exact equality comparison.
///
/// # Arguments
/// * `args[0]` - Search value: `Number` or `Str` (not a Range)
/// * `args[1]` - Lookup range: A `Range` to search in
/// * `args[2]` - Return range: A `Range` to return values from (must have same length as lookup range)
///
/// # Returns
/// * The value from the return range at the matching position (Number, Str, or evaluated Formula)
/// * Empty string `""` if no match found
///
/// # Errors
/// * `LengthError` - If not exactly 3 arguments provided, or if ranges have different lengths
/// * `TypeError` - If first argument is a range, or second/third arguments are not ranges, or formula evaluates to range
///
/// # Example
/// ```text
/// =XLOOKUP("Apple", {0,0}:{0,4}, {1,0}:{1,4})  // Finds "Apple" in row 0, returns value from row 1
/// =XLOOKUP(42, {0,0}:{0,9}, {1,0}:{1,9})  // Finds 42 in first row, returns from second row
/// =XLOOKUP("NotFound", {0,0}:{0,2}, {1,0}:{1,2})  // Returns "" (empty string)
/// // If lookup range: A1="A", B1="B", C1="C" and return range: A2=1, B2=2, C2=3
/// // XLOOKUP("B", A1:C1, A2:C2) returns 2
/// ```
pub fn xlookup(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 3 {
		return Err(Box::new(LengthError("XLOOKUP only accepts 3 arguments".to_string())));
	}

	let [first, second, third] = args.as_slice() else {
		unreachable!()
	};

	let elem = match first {
		FormulaAstInner::Range(_) => return Err(Box::new(TypeError("First argument to XLOOKUP must not be a range".to_string(),))),
		FormulaAstInner::Number(i) => FormulaAstInner::Number(*i),
		FormulaAstInner::Str(str) => FormulaAstInner::Str(str.to_string()),
	};

	let FormulaAstInner::Range(r1) = second else {
		return Err(Box::new(TypeError("Second argument to XLOOKUP must be a range".to_string(),)));
	};

	let FormulaAstInner::Range(r2) = third else {
		return Err(Box::new(TypeError("Third argument to XLOOKUP must be a range".to_string(),)));
	};

	if r1.len() != r2.len() {
		return Err(Box::new(LengthError(format!("Length for first range ({}) is not equal to length of second range ({})", r1.len(), r2.len()))));
	}

	for i in 0..r1.len() {
		let cell = &r1[i].0;
		let value = match &cell.content {
			crate::Content::Formula(formula) => exec._eval(formula, container)?,
			crate::Content::Number(value) => FormulaAstInner::Number(*value),
			crate::Content::Str(str) => FormulaAstInner::Str(str.clone()),
		};

		if matches!(value, FormulaAstInner::Range(_)) {
			return Err(Box::new(TypeError("XLOOKUP range contains a formula that evaluates to a range".to_string())));
		}

		if val_eq(&elem, &value, &super::CmpOperator::Eq) {
			let cell = &r2[i].0;
			let value = match &cell.content {
				crate::Content::Formula(formula) => exec._eval(formula, container)?,
				crate::Content::Number(value) => FormulaAstInner::Number(*value),
				crate::Content::Str(str) => FormulaAstInner::Str(str.clone()),
			};
			return Ok(value);
		}
	}

	Ok(FormulaAstInner::Str("".to_string()))
}

#[cfg(test)]
mod test {

use crate::{Cell, Content, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=XLOOKUP()").unwrap();
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
			content: Content::Str("Potato".to_string()),
			..Default::default()
		};
		m.insert(&vec![0,1,2], cell23).unwrap();

		let ast = q.parse("=XLOOKUP(\"Hello world\", {0,0,0}:{0,0,2}, {0,1,0}:{0,1,2})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 1.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn fails_len() {
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

		let ast = q.parse("=XLOOKUP(\"Hello world\", {0,0,0}:{0,0,2}, {0,1,0}:{0,1,1})").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}
}