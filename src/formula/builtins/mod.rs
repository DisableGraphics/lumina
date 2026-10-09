use std::error::Error;

use crate::{container::Container, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser::FormulaParser}};

/// Contains the now() macro
pub mod now;
/// Contains the countif() macro
pub mod countif;
/// Contains the sumif() macro
pub mod sumif;
/// Contains the averageif() macro
pub mod averageif;
/// Contains the sum() macro
pub mod sum;
/// Contains the count() macro
pub mod count;
/// Contains the counta() macro
pub mod counta;
/// Contains the countblank() macro
pub mod countblank;
/// Contains the product() macro
pub mod product;
/// Contains the average() macro
pub mod average;
/// Contains the min() macro
pub mod min;
/// Contains the max() macro
pub mod max;
/// Contains the abs() macro
pub mod abs;
/// Contains the ceiling() macro
pub mod ceiling;
/// Contains the floor() macro
pub mod floor;
/// Contains the round() macro
pub mod round;
/// Contains the int() macro
pub mod int;
/// Contains the roundup() macro
pub mod roundup;
/// Contains the rounddown() macro
pub mod rounddown;
/// Contains the power() macro
pub mod power;
/// Contains the sqrt() macro
pub mod sqrt;
/// Contains the len() macro
pub mod len;
/// Contains the lower() macro
pub mod lower;
/// Contains the upper() macro
pub mod upper;
/// Contains the left() macro
pub mod left;
/// Contains the right() macro
pub mod right;
/// Contains the middle() macro
pub mod mid;
/// Contains the trim() macro
pub mod trim;
/// Contains the substitute() macro
pub mod substitute;
/// Contains the concatenate() macro
pub mod concat;
/// Contains the isblank() macro
pub mod isblank;
/// Contains the isnumber() macro
pub mod isnumber;
/// Contains the istext() macro
pub mod istext;
/// Contains the iserror() macro
pub mod iserror;
/// Contains the error() macro
pub mod error;
/// Contains the tonumber() macro
pub mod tonumber;
/// Contains the tostring() macro
pub mod tostring;
/// Contains the torange() macro
pub mod torange;
/// Contains the sort() macro
pub mod sort;
/// Contains the unique() macro
pub mod unique;
/// Contains the if() macro
pub mod iff;
/// Contains the and() macro
pub mod and;
/// Contains the or() macro
pub mod or;
/// Contains the not() macro
pub mod not;
/// Contains the iferror() macro
pub mod iferror;
/// Contains the xlookup() macro
pub mod xlookup;
/// Contains the index() macro
pub mod index;
/// Contains the match() macro
pub mod mtch;
enum CmpOperator {
	Le,
	Ge,
	Lt,
	Gt,
	Eq,
	Neq
}

fn val_eq(a: &FormulaAstInner, b: &FormulaAstInner, op: &CmpOperator) -> bool {
	match (a, b) {
		(FormulaAstInner::Number(a), FormulaAstInner::Number(b)) => cmp_num(*a, *b, op),
		(FormulaAstInner::Str(a), FormulaAstInner::Str(b)) => cmp_str(a, b, op),
		(FormulaAstInner::Number(a), FormulaAstInner::Str(b)) | (FormulaAstInner::Str(b), FormulaAstInner::Number(a)) => {
			cmp_str(&a.to_string(), b, op)
		}
		(FormulaAstInner::Range(_), _) | (_, FormulaAstInner::Range(_)) => false,
	}
}

fn ifeval(cell: &str, container: &dyn Container, exec: &FormulaExecutor) -> Result<bool, Box<dyn Error>> {
	let parser = FormulaParser::new();
	let opt: Option<usize> = find_prefixes(cell);
	match opt {
		Some(val) => {
			let expr = &cell[0..val];
			let comp = &cell[val..];

			let exprparsed = parser.parse(format!("={expr}"))?;
			let eval = exec._eval(&exprparsed, container)?;
			
			let (op, right) = get_operator(comp);

			Ok(val_eq(&eval, &right, &op))
		},
		None => {
			let exprparsed = parser.parse(format!("={cell}"))?;
			let eval = exec._eval(&exprparsed, container)?;
			Ok(val_eq(&eval, &FormulaAstInner::Number(0.0), &CmpOperator::Neq))
		}
	}
}

fn cmp_num(a: f64, b: f64, op: &CmpOperator) -> bool {
	match op {
		CmpOperator::Le => a <= b,
		CmpOperator::Ge => a >= b,
		CmpOperator::Lt => a < b,
		CmpOperator::Gt => a > b,
		CmpOperator::Eq => a == b,
		CmpOperator::Neq => a != b,
	}
}

fn cmp_str(a: &str, b: &str, op: &CmpOperator) -> bool {
	match op {
		CmpOperator::Le => a <= b,
		CmpOperator::Ge => a >= b,
		CmpOperator::Lt => a < b,
		CmpOperator::Gt => a > b,
		CmpOperator::Eq => a == b,
		CmpOperator::Neq => a != b,
	}
}

fn tryparse_num(other: &str) -> FormulaAstInner {
	let other = other.trim();

	match other.parse::<f64>() {
		Ok(num) => FormulaAstInner::Number(num),
		Err(_) => {
			let s = other
				.strip_prefix('"')
				.and_then(|s| s.strip_suffix('"'))
				.unwrap_or(other);
			FormulaAstInner::Str(s.to_string())
		}
	}
}

fn strip_prefixes(s: &str) -> (CmpOperator, &str) {
	for (prefix, kind) in [
		(">=", CmpOperator::Ge),
		("<=", CmpOperator::Le),
		(">", CmpOperator::Gt),
		("<", CmpOperator::Lt),
		("!=", CmpOperator::Neq),
		("==", CmpOperator::Eq)
	] {
		if let Some(rest) = s.trim().strip_prefix(prefix) {
			return (kind, rest);
		}
	}

	(CmpOperator::Eq, s)
}

fn find_prefixes(s: &str) -> Option<usize> {
	for prefix in [
		(">="),
		("<="),
		(">"),
		("<"),
		("!="),
		("==")
	] {
		if let Some(rest) = s.find(prefix) {
			return Some(rest);
		}
	}

	None
}


fn get_operator(str: &str) -> (CmpOperator, FormulaAstInner) {
	let pref = strip_prefixes(str);
	let parsed = tryparse_num(pref.1.trim());
	(pref.0, parsed)
}
