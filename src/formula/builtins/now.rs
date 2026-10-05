use std::{error::Error, time::{SystemTime, UNIX_EPOCH}};

use crate::formula::exec::FormulaAstInner;

/// Returns the current date and time as a UNIX timestamp
pub fn now(_args: Vec<FormulaAstInner>) -> Result<FormulaAstInner, Box<dyn Error>> {
	Ok(FormulaAstInner::Number(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64))
}