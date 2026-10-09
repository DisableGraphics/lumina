use std::{error::Error};

use crate::{container::Container, formula::{FormulaError::{LengthError, TypeError}, builtins::{CmpOperator, get_operator, val_eq}, exec::{FormulaAstInner, FormulaExecutor}}};

/// Averages the value of all instances in a range matching an expression
pub fn averageif(args: Vec<FormulaAstInner>, container: &dyn Container, exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("AVERAGEIF only accepts 2 arguments".to_string())));
	}

	let [range, criteria] = args.as_slice() else {
		unreachable!()
	};

	if matches!(criteria, FormulaAstInner::Range(_)) {
		return Err(Box::new(TypeError("Second argument to AVERAGEIF must not be a range".to_string(),)));
	}

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("First argument to AVERAGEIF must be a range".to_string(),)));
	};

	let (op, criteria) = match criteria {
		FormulaAstInner::Range(_) => unreachable!(),
		FormulaAstInner::Number(_) => (CmpOperator::Eq, criteria.clone()),
		FormulaAstInner::Str(s) => get_operator(s)
	};

	if matches!(criteria, FormulaAstInner::Str(_)) {
		return Err(Box::new(TypeError("AVERAGEIF cannot average strings".to_string())))
	}

	let mut count = 0.0;
	let mut v = 0.0;

	for (cell, _) in range {
		let value = match &cell.content {
			crate::Content::Formula(formula) => exec._eval(formula, container)?,
			crate::Content::Number(value) => FormulaAstInner::Number(*value),
			crate::Content::Str(_) => return Err(Box::new(TypeError("AVERAGEIF cannot average strings".to_string()))),
		};

		if matches!(value, FormulaAstInner::Range(_)) {
			return Err(Box::new(TypeError("AVERAGEIF range contains a formula that evaluates to a range".to_string())));
		}

		if matches!(value, FormulaAstInner::Str(_)) {
			return Err(Box::new(TypeError("AVERAGEIF range contains a formula that evaluates to a string".to_string())));
		}

		if val_eq(&value, &criteria, &op) {
			count += 1.0;
			v += match value {
				FormulaAstInner::Number(f) => f,
				_ => unreachable!()
			};
		}
	}

	Ok(FormulaAstInner::Number(v / count))
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AVERAGEIF({0,0,0}:{1,1,1}, 12)").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert!(n.is_nan()),
			_ => assert!(false)
		}
	}

	#[test]
	fn one_instance() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AVERAGEIF({0,0,0}:{1,1,1}, 12)").unwrap();

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
		let ast = q.parse("=AVERAGEIF({0,0,0}:{1,1,1}, 12)").unwrap();

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
	fn fails_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AVERAGEIF({0,0,0}:{1,1,1}, \"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_range() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AVERAGEIF({0,0,0}:{1,1,1}, {0,0,0}:{0,0,1})").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn all_instances_expression() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AVERAGEIF({0,0,0}:{1,1,1}, \">= 2\")").unwrap();

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
			FormulaAstInner::Number(n) => assert_eq!(n, 2.25),
			_ => assert!(false)
		}
	}

	#[test]
	fn all_instances_expression2() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=AVERAGEIF({0,0,0}:{1,1,1}, \"> 2\")").unwrap();

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