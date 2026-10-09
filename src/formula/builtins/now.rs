use std::{error::Error, time::{SystemTime, UNIX_EPOCH}};

use crate::{container::Container, formula::exec::{FormulaAstInner, FormulaExecutor}};

/// NOW - Returns the current date and time as a UNIX timestamp (seconds since epoch).
///
/// Takes no arguments. Returns the current system time as a floating-point number
/// representing seconds since January 1, 1970 (UNIX epoch).
///
/// # Arguments
/// * None (accepts 0 arguments)
///
/// # Returns
/// * `Number` - Current UNIX timestamp as f64 (seconds since epoch)
///
/// # Errors
/// * `LengthError` - If any arguments are provided (currently accepts exactly 0)
///
/// # Example
/// ```text
/// =NOW()  // Returns current timestamp, e.g., 1704067200.0
/// =NOW() / 86400 + 25569  // Convert to Excel serial date format
/// =TONUMBER(NOW())  // Returns numeric timestamp
/// ```
pub fn now(_args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
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