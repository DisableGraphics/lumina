use std::{collections::HashMap, error::Error};

use crate::{Cell, Content, PositionN, container::Container, formula::{FormulaAst, FormulaError::{self, BoundsError, LengthError, MacroDoesNotExist, TypeError}, builtins::now::now}};

/// Public definition of the type that all formulas must implement
pub type FormulaFunction = fn(Vec<FormulaAstInner>) -> Result<FormulaAstInner, Box<dyn Error>>;

/// Executes compiled formulas
pub struct FormulaExecutor {
	fn_table: HashMap<String, FormulaFunction>
}

/// Inner formula AST
#[derive(Debug)]
pub enum FormulaAstInner {
	/// Range
	Range(Vec<(Cell, PositionN)>),
	/// Number
	Number(f64),
	/// String
	Str(String)
}

impl FormulaAstInner {
	fn to_content(self) -> Result<Content, FormulaError> {
		match self {
			Self::Range(_) => Err(LengthError("Can't make a range into a cell".to_string())),
			Self::Number(i) => Ok(Content::Number(i)),
			Self::Str(i) => Ok(Content::Str(i)),
		}
	}
}

fn inner_from_ast(f: &FormulaAst, container: &dyn Container) -> Result<FormulaAstInner, Box<dyn Error>> {
	match f {
		FormulaAst::Range(items) => {
			let cells = container.get_cells_at(items)?;
			Ok(FormulaAstInner::Range(cells.into_iter().zip(items.clone()).collect::<Vec<_>>()))
		},
		FormulaAst::Number(i) => Ok(FormulaAstInner::Number(*i)),
		FormulaAst::Str(str) => Ok(FormulaAstInner::Str(str.clone())),
		other => Err(Box::new(FormulaError::WrongEvaluation(format!("Wrong evaluation: {:?}", other))))
	}
}

fn negate_range(mut src: Vec<(Cell, PositionN)>, exec: &FormulaExecutor, container: &dyn Container) -> Result<FormulaAstInner, Box<dyn Error>> {
	for i in 0..src.len() {
		let p = match &src[i].0.content {
			crate::Content::Formula(formula_ast) => {
				let r = exec.eval(formula_ast, container)?;
				match r {
					FormulaAstInner::Range(_) => return Err(Box::new(FormulaError::TypeError("Cannot negate a range within a range".to_string()))),
					FormulaAstInner::Number(i) => crate::Content::Number(-i),
					FormulaAstInner::Str(_) => return Err(Box::new(FormulaError::TypeError("Cannot negate a string".to_string()))),
				}
			},
			crate::Content::Number(i) => crate::Content::Number(-i),
			crate::Content::Str(_) => return Err(Box::new(FormulaError::TypeError("Cannot negate a string".to_string()))),
		};

		src[i].0.content = p;
	}
	Ok(FormulaAstInner::Range(src))
}

fn op_on_ranges(mut op1: Vec<(Cell, PositionN)>, mut op2: Vec<(Cell, PositionN)>, op: fn(Content, Content, exec: &FormulaExecutor, container: &dyn Container) -> Result<Content, Box<dyn Error>>, exec: &FormulaExecutor, container: &dyn Container) -> Result<FormulaAstInner, Box<dyn Error>> {
	if op1.len() != op2.len() {
		return Err(Box::new(LengthError("Ranges do not match in length".to_string())));
	}
	for i in 0..op1.len() {
		let cnt1 = std::mem::replace(&mut op1[i].0.content, Content::Number(0.0));
		let cnt2 = std::mem::replace(&mut op2[i].0.content, Content::Number(0.0));
		op1[i].0.content = op(cnt1, cnt2, exec, container)?;
	}

	Ok(FormulaAstInner::Range(op1))
}

fn op_on_range_num(mut op1: Vec<(Cell, PositionN)>, num: f64, op: fn(Content, Content, exec: &FormulaExecutor, container: &dyn Container) -> Result<Content, Box<dyn Error>>, exec: &FormulaExecutor, container: &dyn Container) -> Result<FormulaAstInner, Box<dyn Error>> {
	for i in 0..op1.len() {
		let cnt = std::mem::replace(&mut op1[i].0.content, Content::Number(0.0));
		op1[i].0.content = op(cnt, Content::Number(num), exec, container)?;
	}

	Ok(FormulaAstInner::Range(op1))
}

fn op_on_range_num_r(mut op1: Vec<(Cell, PositionN)>, num: f64, op: fn(Content, Content, exec: &FormulaExecutor, container: &dyn Container) -> Result<Content, Box<dyn Error>>, exec: &FormulaExecutor, container: &dyn Container) -> Result<FormulaAstInner, Box<dyn Error>> {
	for i in &mut op1 {
		let cnt = std::mem::replace(&mut i.0.content, Content::Number(0.0));
		i.0.content = op(Content::Number(num), cnt, exec, container)?;
	}

	Ok(FormulaAstInner::Range(op1))
}

fn op_wrapper(cnt1: Content, cnt2: Content, exec: &FormulaExecutor, container: &dyn Container, op: fn(Content, Content) -> Content) -> Result<Content, Box<dyn Error>> {
	if let Content::Number(_) = cnt1 && let Content::Number(_) = cnt2 {
		Ok(op(cnt1, cnt2))
	} else if let Content::Str(_) = cnt1 && let Content::Str(_) = cnt2 {
		Ok(op(cnt2, cnt1))
	} else if let Content::Formula(form1) = &cnt1 && let Content::Formula(form2) = cnt2 {
		let eval1 = exec.eval(form1, container)?;
		let eval2 = exec.eval(&form2, container)?;
		op_wrapper(eval1.to_content()?, eval2.to_content()?, exec, container, op)
	} else if let Content::Formula(form1) = cnt1 {
		let eval1 = exec.eval(&form1, container)?;
		op_wrapper(eval1.to_content()?, cnt2, exec, container, op)
	} else if let Content::Formula(eval2) = cnt2 {
		let eval2 = exec.eval(&eval2, container)?;
		op_wrapper(cnt1, eval2.to_content()?, exec, container, op)
	} else {
		Err(Box::new(TypeError(format!("Can't operate on ranges or on different types: {:?} and {:?}", cnt1, cnt2))))
	}
}

fn add(x: Content, y: Content) -> Content {
	if let Content::Number(x) = x && let Content::Number(y) = y {
		Content::Number(x + y)
	} else {
		Content::Number(0.0)
	}
}

fn sub(x: Content, y: Content) -> Content {
	if let Content::Number(x) = x && let Content::Number(y) = y {
		Content::Number(x - y)
	} else {
		Content::Number(0.0)
	}
}

fn mul(x: Content, y: Content) -> Content {
	if let Content::Number(x) = x && let Content::Number(y) = y {
		Content::Number(x * y)
	} else {
		Content::Number(0.0)
	}
}

fn div(x: Content, y: Content) -> Content {
	if let Content::Number(x) = x && let Content::Number(y) = y {
		Content::Number(x / y)
	} else {
		Content::Number(0.0)
	}
}

fn modu(x: Content, y: Content) -> Content {
	if let Content::Number(x) = x && let Content::Number(y) = y {
		Content::Number(x % y)
	} else {
		Content::Number(0.0)
	}
}

impl Default for FormulaExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn add_vec(mut op1: Vec<usize>, op2: &[usize]) -> Vec<usize> {
	for i in 0..op1.len(){
		op1[i] += op2[i];
	}
	op1
}

fn sub_vec(mut op1: Vec<usize>, op2: &[usize]) -> Vec<usize> {
	for i in 0..op1.len(){
		op1[i] -= op2[i];
	}
	op1
}

impl FormulaExecutor {
	/// Creates a formula executor with all builtin macros
	pub fn new() -> Self {
		Self { 
			fn_table: HashMap::from(
				[
					("now".to_string(), now as FormulaFunction)
				]
			) 
		}
	}

	/// Registers a macro
	pub fn register_function(&mut self, name: &str, f: FormulaFunction) {
		self.fn_table.insert(name.to_string(), f);
	}

	/// Executes a compiled formula in the position position over the container container
	pub fn execute(&self, f: FormulaAst, position: &PositionN, container: &dyn Container) -> Result<(), Box<dyn Error>> {
		let eval = self.eval(&f, container)?;
		match eval {
			FormulaAstInner::Range(mut cells) => {
				if cells.len() > 1 {
					let minimum = cells.iter().min_by(|a, b|{
						a.1.cmp(&b.1)
					});
					if minimum.is_none() {
						return Err(Box::new(FormulaError::BoundsError("No minimum in a range with more than 1 cell".to_string())))
					}
					let minimum = minimum.unwrap().clone();
					for i in cells {
						let destcellpos = add_vec(sub_vec(i.1, &minimum.1), position);
						let cell = container.get_cell_at(&destcellpos)?;
						let cell = match cell {
							Some(mut cell) => {
								cell.display_content = i.0.display_content;
								cell.content = match i.0.content {
									Content::Formula(formula) => Content::Formula(formula),
									other => other
								};
								cell
							},
							None => {
								let mut cell = Cell::default();
								cell.display_content = i.0.display_content;
								cell.content = i.0.content;
								cell
							}
						};
						container.set_cell_at(&destcellpos, cell)?;
					}
				} else if !cells.is_empty() {
					let i = cells.pop().unwrap();
					let cell = container.get_cell_at(position)?;
					let cell = match cell {
						Some(mut cell) => {
							cell.display_content = i.0.display_content;
							cell.content = match i.0.content {
								Content::Formula(formula) => Content::Formula(formula),
								other => other
							};
							cell
						},
						None => {
							let mut cell = Cell::default();
							cell.display_content = i.0.display_content;
							cell.content = i.0.content;
							cell
						}
					};
					container.set_cell_at(position, cell)?;
				} else {
					return Err(Box::new(BoundsError("No cells in range".to_string())))
				}
			},
			FormulaAstInner::Number(i) => {
				let cell = container.get_cell_at(position)?;
				let cell = match cell {
					Some(mut cell) => {
						cell.display_content = i.to_string();
						cell.content = match cell.content {
							Content::Number(_) => Content::Number(i),
							Content::Str(_) => Content::Number(i),
							formula => formula
						};
						cell
					},
					None => {
						let mut cell = Cell::default();
						cell.display_content = i.to_string();
						cell.content = Content::Number(i);
						cell
					}
				};
				container.set_cell_at(position, cell)?;
			},
			FormulaAstInner::Str(i) => {
				let cell = container.get_cell_at(position)?;
				let cell = match cell {
					Some(mut cell) => {
						cell.display_content = i.clone();
						cell.content = match cell.content {
							Content::Number(_) => Content::Str(i),
							Content::Str(_) => Content::Str(i),
							formula => formula
						};
						cell
					},
					None => {
						let mut cell = Cell::default();
						cell.display_content = i.clone();
						cell.content = Content::Str(i);
						cell
					}
				};
				container.set_cell_at(position, cell)?;
			},
		}
		Ok(())
	}

	fn eval(&self, f: &FormulaAst, container: &dyn Container) -> Result<FormulaAstInner, Box<dyn Error>> {
		match f {
			FormulaAst::Neg(formula_ast) => {
				let n1 = self.eval(formula_ast, container)?;
				if let FormulaAstInner::Number(n) = n1 {
					Ok(FormulaAstInner::Number(-n))
				} else if let FormulaAstInner::Range(range) = n1 {
					negate_range(range, self, container)
				} else {
					Err(Box::new(FormulaError::TypeError(format!("{:?} must be of type Number or Cells with Numbers", n1))))
				}
			},
			FormulaAst::Add(formula_ast, formula_ast1) => {
				let n1 = self.eval(formula_ast, container)?;
				let n2 = self.eval(formula_ast1, container)?;

				if let FormulaAstInner::Number(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						Ok(FormulaAstInner::Number(n1 + n2))
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_range_num_r(n2, n1, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, add)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot add {:?} to {:?}", n1, n2))))
					}
				} else if let FormulaAstInner::Range(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						op_on_range_num(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, add)
						}, self, container)
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_ranges(n2, n1, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, add)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot add {:?} to {:?}", n1, n2))))
					}
				} else if let FormulaAstInner::Str(n1) = n1 {
					if let FormulaAstInner::Str(n2) = n2 {
						Ok(FormulaAstInner::Str(n1 + &n2))
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot add {:?} to {:?}", n1, n2))))
					}
				} else {
					Err(Box::new(FormulaError::TypeError(format!("Cannot add {:?} to {:?}", n1, n2))))
				}
			},
			FormulaAst::Sub(formula_ast, formula_ast1) => {
				let n1 = self.eval(formula_ast, container)?;
				let n2 = self.eval(formula_ast1, container)?;

				if let FormulaAstInner::Number(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						Ok(FormulaAstInner::Number(n1 - n2))
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_range_num_r(n2, n1, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, sub)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot substract {:?} from {:?}", n2, n1))))
					}
				} else if let FormulaAstInner::Range(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						op_on_range_num(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, sub)
						}, self, container)
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_ranges(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, sub)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot substract {:?} from {:?}", n2, n1))))
					}
				} else {
					Err(Box::new(FormulaError::TypeError(format!("Cannot substract {:?} from {:?}", n2, n1))))
				}
			},
			FormulaAst::Mul(formula_ast, formula_ast1) => {
				let n1 = self.eval(formula_ast, container)?;
				let n2 = self.eval(formula_ast1, container)?;

				if let FormulaAstInner::Number(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						Ok(FormulaAstInner::Number(n1 * n2))
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_range_num_r(n2, n1, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, mul)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot multiply {:?} by {:?}", n2, n1))))
					}
				} else if let FormulaAstInner::Range(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						op_on_range_num(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, mul)
						}, self, container)
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_ranges(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, mul)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot multiply {:?} by {:?}", n2, n1))))
					}
				} else {
					Err(Box::new(FormulaError::TypeError(format!("Cannot multiply {:?} by {:?}", n2, n1))))
				}
			},
			FormulaAst::Div(formula_ast, formula_ast1) => {
				let n1 = self.eval(formula_ast, container)?;
				let n2 = self.eval(formula_ast1, container)?;

				if let FormulaAstInner::Number(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						Ok(FormulaAstInner::Number(n1 / n2))
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_range_num_r(n2, n1, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, div)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot divide {:?} by {:?}", n2, n1))))
					}
				} else if let FormulaAstInner::Range(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						op_on_range_num(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, div)
						}, self, container)
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_ranges(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, div)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot divide {:?} by {:?}", n2, n1))))
					}
				} else {
					Err(Box::new(FormulaError::TypeError(format!("Cannot divide {:?} by {:?}", n2, n1))))
				}
			},
			FormulaAst::Mod(formula_ast, formula_ast1) => {
				let n1 = self.eval(formula_ast, container)?;
				let n2 = self.eval(formula_ast1, container)?;

				if let FormulaAstInner::Number(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						Ok(FormulaAstInner::Number(n1 % n2))
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_range_num_r(n2, n1, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, modu)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot get modulo of {:?} and {:?}", n2, n1))))
					}
				} else if let FormulaAstInner::Range(n1) = n1 {
					if let FormulaAstInner::Number(n2) = n2 {
						op_on_range_num(n1, n2, |x, y, exec, container| {
							op_wrapper(x, y, exec, container, modu)
						}, self, container)
					} else if let FormulaAstInner::Range(n2) = n2 {
						op_on_ranges(n1, n2, |x, y: Content, exec, container| {
							op_wrapper(x, y, exec, container, modu)
						}, self, container)
					} else {
						Err(Box::new(FormulaError::TypeError(format!("Cannot get modulo of {:?} and {:?}", n2, n1))))
					}
				} else {
					Err(Box::new(FormulaError::TypeError(format!("Cannot get modulo of {:?} and {:?}", n2, n1))))
				}
			},
			FormulaAst::MacroCall(name, formula_asts) => {
				let mut evaluated_arguments = Vec::with_capacity(formula_asts.len());
				for arg in formula_asts {
					evaluated_arguments.push(self.eval(arg, container)?);
				}
				match self.fn_table.get(&name.to_lowercase()) {
					Some(fun) => fun(evaluated_arguments),
					None => Err(Box::new(MacroDoesNotExist(format!("Macro {name} does not exist"))))
				}
			},
			other => inner_from_ast(other, container)
		}
	}
}

#[cfg(test)]
mod test {
	use crate::{Content, container::{Container, mapcontainer::MapContainer}, formula::{exec::{FormulaAstInner, FormulaExecutor}, parser}};

	#[test]
	fn test_eval1() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+12").unwrap();
		let r = FormulaExecutor::new();
		let result = r.eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0+12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_eval2() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7-12").unwrap();
		let r = FormulaExecutor::new();
		let result = r.eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0-12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_eval3() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7*12").unwrap();
		let r = FormulaExecutor::new();
		let result = r.eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0*12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_eval4() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7/12").unwrap();
		let r = FormulaExecutor::new();
		let result = r.eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0/12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_eval5() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7%12").unwrap();
		let r = FormulaExecutor::new();
		let result = r.eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0 % 12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_op_priority() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+12 * 13").unwrap();
		let r = FormulaExecutor::new();
		let result = r.eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_eq!(n, 7.0 + 12.0 * 13.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_call() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=NOW()").unwrap();
		let r = FormulaExecutor::new();
		let result = r.eval(&ast, &m).unwrap();
		match result {
			FormulaAstInner::Number(n) => assert_ne!(n, 0.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_one_cell() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+12").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![0,0,0], &m).unwrap();
		let result = m.get_cell_at(&vec![0,0,0]).unwrap().unwrap();
		match result.content {
			Content::Number(n) => assert_eq!(n, 7.0+12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_another_cell() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+12").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![128,127,126], &m).unwrap();
		let result = m.get_cell_at(&vec![128,127,126]).unwrap().unwrap();
		match result.content {
			Content::Number(n) => assert_eq!(n, 7.0+12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_yet_another_cell() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+12").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![usize::MAX,usize::MAX,usize::MAX], &m).unwrap();
		let result = m.get_cell_at(&vec![usize::MAX,usize::MAX,usize::MAX]).unwrap().unwrap();
		match result.content {
			Content::Number(n) => assert_eq!(n, 7.0+12.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_one_cell_string() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=\"hello\"").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![27,45,13], &m).unwrap();
		let result = m.get_cell_at(&vec![27,45,13]).unwrap().unwrap();
		match result.content {
			Content::Str(n) => assert_eq!(n, "hello"),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_one_cell_range() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=\"hello\"").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![27,45,13], &m).unwrap();

		let ast = q.parse("={27,45,13}").unwrap();
		r.execute(ast, &vec![17,39,23], &m).unwrap();
		let result = m.get_cell_at(&vec![17,39,23]).unwrap().unwrap();
		match result.content {
			Content::Str(n) => assert_eq!(n, "hello"),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_two_cells_int() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+8").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![0,0,0], &m).unwrap();
		let ast = q.parse("=12+3").unwrap();
		r.execute(ast, &vec![0,0,1], &m).unwrap();

		let ast = q.parse("={0,0,0}+{0,0,1}").unwrap();
		r.execute(ast, &vec![0,0,2], &m).unwrap();
		let result = m.get_cell_at(&vec![0,0,2]).unwrap().unwrap();
		match result.content {
			Content::Number(n) => assert_eq!(n, 7.0+8.0+12.0+3.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_vector_int() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+8").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![0,0,0], &m).unwrap();
		let ast = q.parse("=12+3").unwrap();
		r.execute(ast, &vec![0,0,1], &m).unwrap();

		let ast = q.parse("=10+6").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![0,1,0], &m).unwrap();
		let ast = q.parse("=12+6").unwrap();
		r.execute(ast, &vec![0,1,1], &m).unwrap();

		let ast = q.parse("={0,0,0}:{0,0,1}+{0,1,0}:{0,1,1}").unwrap();
		r.execute(ast, &vec![0,0,2], &m).unwrap();
		let result = m.get_cell_at(&vec![0,0,2]).unwrap().unwrap();
		match result.content {
			Content::Number(n) => assert_eq!(n, 7.0+8.0+10.0+6.0),
			_ => assert!(false)
		}
		let result = m.get_cell_at(&vec![0,0,3]).unwrap().unwrap();
		match result.content {
			Content::Number(n) => assert_eq!(n, 12.0+3.0+12.0+6.0),
			_ => assert!(false)
		}
	}

	#[test]
	fn test_exec_vector_fail() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=7+8").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![0,0,0], &m).unwrap();
		let ast = q.parse("=12+3").unwrap();
		r.execute(ast, &vec![0,0,1], &m).unwrap();

		let ast = q.parse("=10+6").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![0,1,0], &m).unwrap();
		let ast = q.parse("=12+6").unwrap();
		r.execute(ast, &vec![0,1,1], &m).unwrap();

		let ast = q.parse("={0,0,0}:{0,0,2}+{0,1,0}:{0,1,1}").unwrap();
		assert!(r.execute(ast, &vec![0,0,2], &m).is_err());
	}

	#[test]
	fn test_exec_non_recursive() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("={0,0,1}+1").unwrap();
		let r = FormulaExecutor::new();
		r.execute(ast, &vec![0,0,0], &m).unwrap();
		let ast = q.parse("={0,0,0}+1").unwrap();
		r.execute(ast, &vec![0,0,1], &m).unwrap();
	}
}