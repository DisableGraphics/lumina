use std::{cmp::Ordering::Equal, collections::{BTreeMap, BTreeSet}, error::Error};

struct Wrapper<T>(T);

impl<T: PartialEq> PartialEq for Wrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: PartialOrd> PartialOrd for Wrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<T: PartialEq> Eq for Wrapper<T> {}
impl<T: PartialOrd> Ord for Wrapper<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap_or(Equal)
	}
}

use crate::{Cell, PositionN, container::Container, formula::{FormulaError::{LengthError, TypeError}, exec::{FormulaAstInner, FormulaExecutor}}};

fn gather_slices(range: &Vec<(Cell, PositionN)>, axis: usize) -> BTreeMap<usize, Vec<(Cell, PositionN)>> {
    let mut slices = BTreeMap::new();

    for (cell, p) in range {
        let key = p[axis];

        slices
            .entry(key)
            .or_insert_with(Vec::new)
            .push((cell.clone(), p.clone()));
    }

    slices
}

/// Returns the UNIQUEolute of a range
pub fn unique(args: Vec<FormulaAstInner>, _container: &dyn Container, _exec: &FormulaExecutor) -> Result<FormulaAstInner, Box<dyn Error>> {
	if args.len() != 2 {
		return Err(Box::new(LengthError("UNIQUE only accepts 2 arguments".to_string())));
	}

	let [range, int] = args.as_slice() else {
		unreachable!()
	};

	let FormulaAstInner::Range(range) = range else {
		return Err(Box::new(TypeError("First argument to UNIQUE must be a range".to_string(),)));
	};

	let FormulaAstInner::Number(axis) = int else {
		return Err(Box::new(TypeError("First argument to UNIQUE must be a number".to_string(),)));
	};

	let axis = *axis as usize;

	let mut seen = BTreeSet::new();
	let mut uniq_slic = Vec::new();
	let slices = gather_slices(&range, axis);
	
	for (_p, slice) in slices {
		let key: Vec<Cell> = slice
			.iter()
			.map(|(cell, _)| cell.clone())
			.collect();
	
		if seen.insert(Wrapper(key)) {
			uniq_slic.push(slice);
		}
	}

	let mut range = Vec::new();

	for (new_axis_index, slice) in uniq_slic.iter().enumerate() {
		for (cell, position) in slice {
			let mut new_position = position.clone();
			new_position[axis] = new_axis_index;
			
			range.push((cell.clone(), new_position));
		}
	}
	Ok(FormulaAstInner::Range(range))
	
}

#[cfg(test)]
mod test {
	use crate::{Cell, container::{Container, mapcontainer::MapContainer}, formula::{exec::FormulaExecutor, parser}};

	#[test]
	fn no_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=UNIQUE({0,0,0}:{1,1,1}, 2)").unwrap();

		let r = FormulaExecutor::new();
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		assert_eq!(m.get_cell_at(&vec![2,2,2]).unwrap().unwrap().content, crate::Content::Number(0.0));
	}

	#[test]
	fn one_instance() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=UNIQUE({0,0,0}:{1,1,1}, 0)").unwrap();

		let mut def = Cell::default();
		def.content = crate::Content::Number(-12.0);

		m.insert(&vec![0,0,1], def).unwrap();

		let r = FormulaExecutor::new();
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		assert_eq!(m.get_cell_at(&vec![2,2,2]).unwrap().unwrap().content, crate::Content::Number(0.0));
		assert_eq!(m.get_cell_at(&vec![2,2,3]).unwrap().unwrap().content, crate::Content::Number(-12.0));
	}

	#[test]
	fn all_instances() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=UNIQUE({0,0,0}:{1,1,1}, 0)").unwrap();

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
		r.execute(ast, &vec![2,2,2], &m).unwrap();
		assert_eq!(m.get_cell_at(&vec![2,2,2]).unwrap().unwrap().content, crate::Content::Number(12.0));
		assert_ne!(m.get_cell_at(&vec![2,2,3]).unwrap().unwrap().content, crate::Content::Number(0.0));
	}

	#[test]
	fn fails_str() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=UNIQUE(\"potato\")").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}

	#[test]
	fn fails_num() {
		let m = MapContainer::new(3);
		let q = parser::FormulaParser::new();
		let ast = q.parse("=UNIQUE(1)").unwrap();
		let r = FormulaExecutor::new();
		assert!(r._eval(&ast, &m).is_err());
	}
}