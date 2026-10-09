use std::{error::Error, time::{SystemTime, UNIX_EPOCH}};

use crate::{container::Container, formula::exec::{FormulaAstInner, FormulaExecutor}};

/// Returns the current date and time as a UNIX timestamp
pub fn now(_args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	// Oh no, this will start getting less precision in 285 million years!!! HORRIBLE!!!!
	Ok(FormulaAstInner::Number(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64))
}

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};

use crate::{container::mapcontainer::MapContainer, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn not_empty() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=NOW()").unwrap();
		let r = FormulaExecutor::new();
		let result = r._eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_ne!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn actual_correct_inner() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=NOW()").unwrap();
		let r = FormulaExecutor::new();
		let mut correct = false;
		// try up to 5 times to prevent a possible race
		for _ in 0..5 {
			let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
			let result = r._eval(&ast, &m).unwrap();
			match result {
				FormulaAstInner::Number(n) => if n == now {
					correct = true;
					break;
				},
				_ => assert!(false)
			}
		}
		assert!(correct)
	}
}