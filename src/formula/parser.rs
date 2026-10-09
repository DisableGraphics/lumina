use crate::formula::{FormulaAst::self, FormulaError};
use chumsky::prelude::*;

/// Parses a String into a FormulaAst
pub struct FormulaParser {
	
}

fn expand(origin: &[usize], dest: &[usize]) -> Vec<Vec<usize>> {
	let mut result = vec![vec![]];
    
    for (o, d) in origin.iter().zip(dest.iter()) {
        let mut new_result = Vec::new();
        for current in result {
            for i in *o..=*d {
                let mut row = current.clone();
                row.push(i);
                new_result.push(row);
            }
        }
        result = new_result;
    }
    
    result
}

fn parser<'src>() -> impl Parser<'src, &'src str, FormulaAst, extra::Err<Rich<'src, char>>> {
	let id = text::unicode::ident().map(|s: &str| s.to_string()).padded();
	let int = text::int(10).map(|s: &str| s.parse().unwrap());
	let comma = just(',').padded();
	let op = |c| just(c).padded();
	let position = int.separated_by(comma).at_least(1).collect::<Vec<usize>>().delimited_by(just('{'), just('}'));
	recursive(|pr| {	
		let range = choice((
			position
				.then_ignore(just(':'))
				.then(position)
				.map(|(orig, dest)| FormulaAst::Range(expand(&orig, &dest))),
			position
				.map(|x| FormulaAst::Range(expand(&x, &x))),
			position
				.separated_by(just(';').padded())
				.at_least(1)
				.collect::<Vec<_>>()
				.map(FormulaAst::Range),
		));
	
		let number = choice((
			int
				.then_ignore(just('.'))
				.then(int)
				.map(|(p1, p2)| {
					if p2 != 0 {
						let dplaces = (p2 as f64).log10() as i32 + 1;
						FormulaAst::Number(
							p1 as f64 + p2 as f64 / 10_f64.powi(dplaces),
						)
					} else {
						FormulaAst::Number(p1 as f64)
					}
				}),
		
			int
				.map(|x| FormulaAst::Number(x as f64)),
		));
		
	
		let string = choice((
				just('\\')
					.ignore_then(just('"'))
					.to('"'),
				just('"')
					.ignore_then(just('"'))
					.to('"'),
				none_of('"'),
			))
			.repeated()
			.collect::<String>()
			.delimited_by(just('"'), just('"'))
			.map(FormulaAst::Str);

	
		let args = pr
			.clone()
			.separated_by(comma)
			.collect::<Vec<_>>();
		
		let call = id
			.then(
				args.delimited_by(
					just('(').padded(),
					just(')').padded(),
				)
			)
			.map(|(f, args)| FormulaAst::MacroCall(f, args));
		
		let atom = choice((
			string,
			call,
			number,
			range,
			pr.clone().delimited_by(just('('), just(')')),
		))
		.padded();
	
		let unary = op('-')
			.repeated()
			.foldr(atom, |_, rhs| FormulaAst::Neg(Box::new(rhs)));
	
		let product = unary.clone().foldl(
			choice((
				op('*').to(FormulaAst::Mul as fn(_, _) -> _),
				op('/').to(FormulaAst::Div as fn(_, _) -> _),
				op('%').to(FormulaAst::Mod as fn(_, _) -> _),
			))
			.then(unary)
			.repeated(),
			|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
		);
	
		product.clone().foldl(
			choice((
				op('+').to(FormulaAst::Add as fn(_, _) -> _),
				op('-').to(FormulaAst::Sub as fn(_, _) -> _),
			))
			.then(product)
			.repeated(),
			|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
		)
	})
	
}

impl Default for FormulaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl FormulaParser {
	/// Creates a new empty FormulaParser
	pub fn new() -> Self {
		Self {  }
	}

	/// Parses a possible formula into an AST
	pub fn parse<P: AsRef<str>>(&self, command: P) -> Result<FormulaAst, FormulaError> {
		let strc = command.as_ref();
		if let Some(c) = strc.strip_prefix("=") {
			let parser = parser();
			let r = parser.then_ignore(end()).parse(c);

			match r.into_result() {
				Ok(o) => Ok(o),
				Err(e) => {
					let errors = e.into_iter().map(|x| 
						format!("{}: {x}", x.span())
					).collect::<Vec<String>>();
					Err(FormulaError::SyntaxError(errors.join("\n")))
				}
			}
		} else {
			Err(FormulaError::SyntaxError("Missing '=' at 0".to_string()))
		}
	}
}

#[cfg(test)]
mod test {
	use crate::formula::{FormulaAst, parser::FormulaParser};

	#[test]
	fn parse_simple() {
		let p = FormulaParser::new();
		match p.parse("=SUM(3,7)") {
			Ok(_) => {},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			} 
		}
	}

	#[test]
	fn parse_string() {
		let p = FormulaParser::new();

		match p.parse(r#"=SUM("string")"#) {
			Ok(_) => {},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			}
		}
	}

	#[test]
	fn parse_expr() {
		let p = FormulaParser::new();
		match p.parse("=7+7") {
			Ok(_) => {},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			} 
		}
	}

	#[test]
	fn parse_recursive() {
		let p = FormulaParser::new();
		match p.parse("=SUM(SUM(SUM(SUM(\"string\"))))") {
			Ok(_) => {},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			} 
		}
	}

	#[test]
	fn parse_0args() {
		let p = FormulaParser::new();
		match p.parse("=SUM()") {
			Ok(_) => {},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			} 
		}
	}

	#[test]
	fn parse_massive_expression() {
		let p = FormulaParser::new();
		match p.parse("=SUM({12,3,4}:{7,36,9}) * 7 + 73 % 12 * NOW() + {1,2,3}") {
			Ok(_) => {},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			} 
		}
	}

	#[test]
	fn parse_range() {
		let p = FormulaParser::new();
		match p.parse("={1,2,3,4,5}") {
			Ok(_) => {},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			} 
		}
	}

	#[test]
	fn parse_range2() {
		let p = FormulaParser::new();
		match p.parse("={1,2,3,4,5}") {
			Ok(root) => {
				let eq = FormulaAst::Range(vec![vec![1 as usize,2,3,4,5]]);
				assert_eq!(root, eq);
			},
			Err(e) => {
				eprintln!("{e}");
				assert!(false);
			}
		}
	}

	#[test]
	fn parse_error_range() {
		let p = FormulaParser::new();
		assert!(p.parse("={1,2,3,4,}").is_err());
	}

	#[test]
	fn parse_error_range2() {
		let p = FormulaParser::new();
		assert!(p.parse("={1,2,3,4").is_err());
	}

	#[test]
	fn parse_error_range3() {
		let p = FormulaParser::new();
		assert!(p.parse("=1,2,3,4}").is_err());
	}

	#[test]
	fn parse_error_expr() {
		let p = FormulaParser::new();
		assert!(p.parse("=1*").is_err());
	}

	#[test]
	fn parse_error_call() {
		let p = FormulaParser::new();
		assert!(p.parse("=SUM)").is_err());
	}

	#[test]
	fn parse_error_call2() {
		let p = FormulaParser::new();
		assert!(p.parse("=(SUM)").is_err());
	}

	#[test]
	fn parse_error_call3() {
		let p = FormulaParser::new();
		assert!(p.parse("=SUM(").is_err());
	}

	#[test]
	fn parse_error_string() {
		let p = FormulaParser::new();
		assert!(p.parse("=\"h").is_err());
	}

	#[test]
	fn parse_error_string2() {
		let p = FormulaParser::new();
		assert!(p.parse("=h\"").is_err());
	}

	#[test]
	fn parse_error_neg() {
		let p = FormulaParser::new();
		assert!(p.parse("=-").is_err());
	}

	#[test]
	fn parse_error_operator1() {
		let p = FormulaParser::new();
		assert!(p.parse("=7+").is_err());
	}

	#[test]
	fn parse_error_operator2() {
		let p = FormulaParser::new();
		assert!(p.parse("=+7").is_err());
	}

	#[test]
	fn parse_error_operator3() {
		let p = FormulaParser::new();
		assert!(p.parse("=*7").is_err());
	}

	#[test]
	fn parse_error_operator4() {
		let p = FormulaParser::new();
		assert!(p.parse("=7*").is_err());
	}

	#[test]
	fn parse_error_operator5() {
		let p = FormulaParser::new();
		assert!(p.parse("=7-").is_err());
	}

	#[test]
	fn parse_error_operator6() {
		let p = FormulaParser::new();
		assert!(p.parse("=/7").is_err());
	}

	#[test]
	fn parse_error_operator7() {
		let p = FormulaParser::new();
		assert!(p.parse("=7/").is_err());
	}

	#[test]
	fn parse_error_operator8() {
		let p = FormulaParser::new();
		assert!(p.parse("=%7").is_err());
	}

	#[test]
	fn parse_error_operator9() {
		let p = FormulaParser::new();
		assert!(p.parse("=7%").is_err());
	}
}