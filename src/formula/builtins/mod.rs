use std::error::Error;

use crate::{container::Container, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser::FormulaParser}};

/// Built-in spreadsheet functions (macros).
///
/// Each function follows the signature:
/// ```rust,ignore
/// pub fn function_name(
///     args: Vec<FormulaAstInner>,
///     container: &dyn Container,
///     exec: &FormulaExecutor
/// ) -> Result<FormulaAstInner, Box<dyn Error>> { todo!() }
/// ```
///
/// Functions are registered in the `MACROS` map at the bottom of this file.
///
/// # Argument Types
///
/// - `FormulaAstInner::Range(Vec<PositionN>)` - A cell range reference
/// - `FormulaAstInner::Number(f64)` - A numeric value
/// - `FormulaAstInner::Str(String)` - A string value
///
/// # Return Values
///
/// Functions return `FormulaAstInner`:
/// - `Number(f64)` for scalar results
/// - `Range(Vec<(Cell, PositionN)>)` for range results (element-wise operations)
/// - `Str(String)` for string results

/// Contains the now() macro - returns current UNIX timestamp
pub mod now;
/// Contains the countif() macro - counts cells matching criteria
pub mod countif;
/// Contains the sumif() macro - sums cells matching criteria
pub mod sumif;
/// Contains the averageif() macro - averages cells matching criteria
pub mod averageif;
/// Contains the sum() macro - sums all numbers in a range
pub mod sum;
/// Contains the count() macro - counts numeric cells in a range
pub mod count;
/// Contains the counta() macro - counts non-empty cells in a range
pub mod counta;
/// Contains the countblank() macro - counts empty cells in a range
pub mod countblank;
/// Contains the product() macro - multiplies all numbers in a range
pub mod product;
/// Contains the average() macro - arithmetic mean of a range
pub mod average;
/// Contains the min() macro - minimum value in a range
pub mod min;
/// Contains the max() macro - maximum value in a range
pub mod max;
/// Contains the abs() macro - absolute value of each cell in a range
pub mod abs;
/// Contains the ceiling() macro - ceiling of each cell in a range
pub mod ceiling;
/// Contains the floor() macro - floor of each cell in a range
pub mod floor;
/// Contains the round() macro - rounds each cell to specified decimal places
pub mod round;
/// Contains the int() macro - integer part (truncates) of each cell in a range
pub mod int;
/// Contains the roundup() macro - rounds up each cell to specified decimal places
pub mod roundup;
/// Contains the rounddown() macro - rounds down each cell to specified decimal places
pub mod rounddown;
/// Contains the power() macro - raises each cell to a power
pub mod power;
/// Contains the sqrt() macro - nth root of each cell in a range
pub mod sqrt;
/// Contains the len() macro - string length
pub mod len;
/// Contains the lower() macro - converts string to lowercase
pub mod lower;
/// Contains the upper() macro - converts string to uppercase
pub mod upper;
/// Contains the left() macro - left N characters of a string
pub mod left;
/// Contains the right() macro - right N characters of a string
pub mod right;
/// Contains the middle() macro - middle N characters of a string
pub mod mid;
/// Contains the trim() macro - trims whitespace from a string
pub mod trim;
/// Contains the substitute() macro - replaces all occurrences in a string
pub mod substitute;
/// Contains the concatenate() macro - concatenates all strings in a range
pub mod concat;
/// Contains the isblank() macro - checks if cells are empty
pub mod isblank;
/// Contains the isnumber() macro - checks if cells contain numbers
pub mod isnumber;
/// Contains the istext() macro - checks if cells contain strings
pub mod istext;
/// Contains the iserror() macro - checks if cells contain errors
pub mod iserror;
/// Contains the error() macro - returns an error value
pub mod error;
/// Contains the tonumber() macro - parses string to number
pub mod tonumber;
/// Contains the tostring() macro - converts number to string
pub mod tostring;
/// Contains the torange() macro - converts a number to a single-cell range
pub mod torange;
/// Contains the sort() macro - sorts a range of values
pub mod sort;
/// Contains the unique() macro - returns unique values from a range
pub mod unique;
/// Contains the if() macro - ternary conditional
pub mod iff;
/// Contains the and() macro - logical AND
pub mod and;
/// Contains the or() macro - logical OR
pub mod or;
/// Contains the not() macro - logical NOT
pub mod not;
/// Contains the iferror() macro - catches errors in formula evaluation
pub mod iferror;
/// Contains the xlookup() macro - lookup value and return corresponding result
pub mod xlookup;
/// Contains the index() macro - returns cell at relative position in range
pub mod index;
/// Contains the match() macro - finds position of value in range
pub mod mtch;

/// Comparison operators for conditional functions (COUNTIF, SUMIF, etc.)
enum CmpOperator {
    Le,
    Ge,
    Lt,
    Gt,
    Eq,
    Neq
}

/// Compares two FormulaAstInner values using the given operator.
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

/// Evaluates a conditional expression string against the container.
/// Used by COUNTIF, SUMIF, AVERAGEIF for criteria like ">5" or "text".
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

/// Attempts to parse a string as a number, falls back to string.
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

/// Strips comparison operator prefixes (>=, <=, >, <, !=, ==) from a string.
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

/// Finds the position of a comparison operator in a string.
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

/// Parses a comparison operator and its right-hand operand.
fn get_operator(str: &str) -> (CmpOperator, FormulaAstInner) {
    let pref = strip_prefixes(str);
    let parsed = tryparse_num(pref.1.trim());
    (pref.0, parsed)
}

/// Map of all built-in function names to their implementations.
/// Used by the formula executor to dispatch macro calls.
pub fn get_macros() -> std::collections::HashMap<&'static str, fn(Vec<FormulaAstInner>, &dyn Container, &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>>> {
    use std::collections::HashMap;
    type MacroFn = fn(Vec<FormulaAstInner>, &dyn Container, &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>>;
    let mut map: HashMap<&'static str, MacroFn> = HashMap::new();

    // Math
    map.insert("SUM", sum::sum);
    map.insert("AVERAGE", average::average);
    map.insert("PRODUCT", product::product);
    map.insert("MIN", min::min);
    map.insert("MAX", max::max);
    map.insert("ABS", abs::abs);
    map.insert("CEILING", ceiling::ceiling);
    map.insert("FLOOR", floor::floor);
    map.insert("ROUND", round::round);
    map.insert("INT", int::int);
    map.insert("ROUNDUP", roundup::roundup);
    map.insert("ROUNDDOWN", rounddown::rounddown);
    map.insert("POWER", power::power);
    map.insert("SQRT", sqrt::sqrt);

    // String
    map.insert("CONCAT", concat::concat);
    map.insert("LEFT", left::left);
    map.insert("RIGHT", right::right);
    map.insert("MID", mid::mid);
    map.insert("LEN", len::len);
    map.insert("TRIM", trim::trim);
    map.insert("UPPER", upper::upper);
    map.insert("LOWER", lower::lower);
    map.insert("SUBSTITUTE", substitute::substitute);
    map.insert("TOSTRING", tostring::tostring);
    map.insert("TONUMBER", tonumber::tonumber);

    // Logical
    map.insert("IF", iff::iff);
    map.insert("AND", and::and);
    map.insert("OR", or::or);
    map.insert("NOT", not::not);
    map.insert("IFERROR", iferror::iferror);
    map.insert("ISERROR", iserror::iserror);
    map.insert("ISBLANK", isblank::isblank);
    map.insert("ISNUMBER", isnumber::isnumber);
    map.insert("ISTEXT", istext::istext);

    // Lookup
    map.insert("XLOOKUP", xlookup::xlookup);
    map.insert("INDEX", index::index);
    map.insert("MATCH", mtch::mtch);

    // Statistical
    map.insert("COUNT", count::count);
    map.insert("COUNTA", counta::counta);
    map.insert("COUNTBLANK", countblank::countblank);
    map.insert("COUNTIF", countif::countif);
    map.insert("SUMIF", sumif::sumif);
    map.insert("AVERAGEIF", averageif::averageif);

    // Data
    map.insert("SORT", sort::sort);
    map.insert("UNIQUE", unique::unique);
    map.insert("TORANGE", torange::torange);

    // Time
    map.insert("NOW", now::now);

    // Error
    map.insert("ERROR", error::error);

    map
}