use std::error::Error;

use crate::{container::Container, formula::{FormulaError::{LengthError, RuntimeError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}, vec_sub}};

/// Returns the current date and time as a UNIX timestamp
pub fn index(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() < 2 {
		return Err(Box::new(LengthError("INDEX accepts at least 2 arguments".to_string())));
	} else if !(args.len() == 1 + container.get_axes() || args.len() == 2) {
		return Err(Box::new(LengthError("INDEX can't be called with more optional arguments than dimensions".to_string())));
	}

	let FormulaAstInner::Range(r) = &args[0] else {
		return Err(Box::new(TypeError("First argument to INDEX must be a range".to_string(),)));
	};

	let Some(minimum) = r.iter().min_by(|a, b| a.1.cmp(&b.1)) else {
		return Err(Box::new(RuntimeError("INDEX did not find a minimum position".to_string(),)));
	};

	let other: Vec<usize> = match &args[1] {
		FormulaAstInner::Str(_) => return Err(Box::new(TypeError("Positions must evaluate to an argument".to_string()))),
		FormulaAstInner::Number(_) => {
			args[1..].iter()
			.map(|x| {
				match x {
					FormulaAstInner::Range(_) => Err(Box::new(TypeError("Positions must evaluate to an argument".to_string()))),
					FormulaAstInner::Number(i) => Ok(*i as usize),
					FormulaAstInner::Str(_) => Err(Box::new(TypeError("Positions must evaluate to an argument".to_string()))),
				}
			})
			.collect::<Result<_, _>>()?
		},
		FormulaAstInner::Range(r) => {
			if r.len() != 1 {
				return Err(Box::new(LengthError("INDEX range must be for one element only".to_string())));
			} else {
				r[0].1.clone()
			}
		}
	};

	let Some((cell, _)) = r.iter().find(|(_, pos)| {
		vec_sub(pos, &minimum.1) == other
	}) else {
		return Err(Box::new(TypeError("Positions must evaluate to an argument".to_string())))
	};

	let value = match &cell.content {
		crate::Content::Formula(formula) => exec._eval(formula, container)?,
		crate::Content::Number(value) => FormulaAstInner::Number(*value),
		crate::Content::Str(str) => FormulaAstInner::Str(str.clone()),
	};

	if matches!(value, FormulaAstInner::Range(_)) {
		return Err(Box::new(TypeError("INDEX range contains a formula that evaluates to a range".to_string())));
	}

	Ok(value)
}

#[cfg(test)]
mod test {

use crate::{Cell, Content, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn fails_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=INDEX()").unwrap();
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

		let ast = q.parse("=INDEX({0,0,1}:{0,1,2}, 0, 1, 1)").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 5.0),
			_ => assert!(false)
		}
	}

	
	#[test]
	fn correct_len_range() {
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

		let ast = q.parse("=INDEX({0,0,1}:{0,1,2}, {0, 1, 1})").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 5.0),
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

		let ast = q.parse("=INDEX({0,0,0}:{0,1,2}, 1)").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}
}