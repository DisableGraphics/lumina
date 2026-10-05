use std::{collections::HashMap, sync::{Arc, RwLock}};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Cell, PositionN, container::{Container, ContainerError::{CoordinateMismatch, InvalidAxesNumber, OutOfBounds}, RetError}};

#[derive(Serialize, Deserialize, Debug)]
struct InnerContainer {
	cont: HashMap<PositionN, Cell>,
	naxes: usize
}

impl InnerContainer {
	fn new(axes: usize) -> Self {
		Self {
			cont: HashMap::new(),
			naxes: axes
		}
	}
}

/// Container that uses a HashMap as a backing store
pub struct MapContainer {
	inner: Arc<RwLock<InnerContainer>>,
	naxes: usize // READ ONLY TO MAKE LOOKUPS FASTER
}

impl MapContainer {
	/// Create new MapContainer with some axes
	pub fn new(axes: usize) -> Self {
		Self { inner: Arc::new(RwLock::new(InnerContainer::new(axes))), naxes: axes }
	}
}

impl Iterator for MapContainer {
	type Item = (Vec<usize>, Cell);
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.read().unwrap().cont.iter().next().map(|x| (x.0.clone(), x.1.clone()))
	}
}

impl<'a> Container for MapContainer {
	fn insert(&self, p: &PositionN, c: Cell) -> Result<(), RetError> {
		if p.len() != self.naxes {
			return Err(Box::new(CoordinateMismatch(format!("Wrong coordinate length when inserting: {} (requested) is not {} (actual)", p.len(), self.naxes))));
		}
		let mut writer = self.inner.write().unwrap();
		writer.cont.insert(p.clone(), c);
		Ok(())
	}

	fn remove(&self, p: &PositionN) -> Result<(), RetError> {
		if p.len() != self.naxes {
			return Err(Box::new(CoordinateMismatch(format!("Wrong coordinate length when deleting: {} (requested) is not {} (actual)", p.len(), self.naxes))));
		}
		let mut writer = self.inner.write().unwrap();
		writer.cont.remove(p);

		Ok(())
	}

	fn get_cell_at(&self, p: &PositionN) -> Result<Option<Cell>, RetError> {
		if p.len() != self.naxes {
			return Err(Box::new(CoordinateMismatch(format!("Wrong coordinate length when getting cell: {} (requested) is not {} (actual)", p.len(), self.naxes))));
		}
		let reader = self.inner.read().unwrap();
		Ok(reader.cont.get(p).cloned())
	}

	fn get_cells_at(&self, p: &Vec<PositionN>) -> Result<Vec<Cell>, RetError> {
		let reader = self.inner.read().unwrap();

		let mut ret = vec![];
		for p in p {
			if p.len() != self.naxes {
				return Err(Box::new(CoordinateMismatch(format!("Wrong coordinate length when getting cell: {} (requested) is not {} (actual)", p.len(), self.naxes))));
			}
			if let Some(o) = reader.cont.get(p) {
				ret.push(o.clone());
			} else {
				ret.push(Cell::default())
			}
		}
		Ok(ret)
	}

	fn set_axes(&mut self, naxes: usize) -> Result<(), RetError> {
		if naxes > 0 {
			let prevaxes = self.naxes;
			self.naxes = naxes;
			if naxes > prevaxes {
				let mut data = self.inner.write().unwrap();
				data.naxes = naxes;
				let dcont = std::mem::take(&mut data.cont);
				for mut d in dcont {
					d.0.resize(naxes, 0);
					data.cont.insert(d.0, d.1);
				}
			} else if naxes < prevaxes {
				let mut data = self.inner.write().unwrap();
				data.naxes = naxes;
				let dcont = std::mem::take(&mut data.cont);
				for mut d in dcont {
					if d.0[naxes..prevaxes].iter().all(|x| *x == 0) {
						d.0.truncate(naxes);
						data.cont.insert(d.0, d.1);
					}
				}
			}
			Ok(())
		} else {
			Err(Box::new(InvalidAxesNumber("Can't have a container with 0 axes".to_string())))
		}
	}

	fn remove_axis(&mut self, axispos: usize, retain_coord: Option<usize>) -> Result<(), RetError> {
		if self.naxes <= 1 {
			return Err(Box::new(InvalidAxesNumber("Can't have a container with 0 axes".to_string())));
		}
		if axispos >= self.naxes {
			return Err(Box::new(OutOfBounds(format!("{axispos} is larger than maximum dimension: {}", self.naxes))));
		}
		self.naxes -= 1;
		let mut data = self.inner.write().unwrap();
		data.naxes-=1;
		let dcont = std::mem::take(&mut data.cont);
		for mut d in dcont {
			if d.0[axispos] == retain_coord.unwrap_or(0) {
				d.0.remove(axispos);
				data.cont.insert(d.0, d.1);
			}
		}
		Ok(())
	}
	
	fn set_cell_at(&self, p: &PositionN, nc: Cell) -> Result<(), RetError> {
		self.insert(p, nc)
	}
	
	fn get_axes(&self) -> usize {
		self.naxes
	}
	
	fn get_cells_at_axis(&self, p: &[super::AnyPosition]) -> Result<Vec<Cell>, RetError> {
		if p.len() != self.naxes {
			return Err(Box::new(CoordinateMismatch(format!("Wrong coordinate length when getting cell: {} (requested) is not {} (actual)", p.len(), self.naxes))));
		}
		let reader = self.inner.read().unwrap();
		let mut ret = vec![];
		'life: for i in &reader.cont {
			for j in 0..i.0.len() {
				match p[j] {
					super::AnyPosition::Any => {},
					super::AnyPosition::Position(us) => {
						match us == i.0[j] {
							true => {},
							false => { continue 'life }
						}
					},
				}
			}
			ret.push(i.1.clone());
		}
		Ok(ret)
	}
	
	fn get_n_cells(&self) -> usize {
		let reader = self.inner.read().unwrap();
		reader.cont.len()
	}
}

impl<'de> Deserialize<'de> for MapContainer {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let inner: InnerContainer = InnerContainer::deserialize(deserializer)?;
		let axes = inner.naxes;
		Ok(Self {
			inner: Arc::new(RwLock::new(inner)),
			naxes: axes,
		})
	}
}

impl Serialize for MapContainer {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let inner = self.inner.read().unwrap();
		inner.serialize(serializer)
	}
}


#[cfg(test)]
mod test {
	use crate::{Cell, Color, container::{AnyPosition, Container, mapcontainer::MapContainer}};

	#[test]
	fn insertion() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		assert!(c.insert(&vec![1,2,3], cell).is_ok());
	}

	#[test]
	fn wrong_coords_insert() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		assert!(c.insert(&vec![1,2], cell).is_err());
	}

	#[test]
	fn getting_at() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		let checkcell = cell.clone();
		assert!(c.insert(&vec![1,2,3], cell).is_ok());
		assert_eq!(c.get_cell_at(&vec![1,2,3]).unwrap().unwrap(), checkcell);
	}

	#[test]
	fn wrong_coords_get() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		assert!(c.insert(&vec![1,2,3], cell).is_ok());
		assert!(c.get_cell_at(&vec![1,2]).is_err());
	}

	#[test]
	fn setting_at() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		let mut checkcell = cell.clone();
		assert!(c.insert(&vec![1,2,3], cell).is_ok());
		assert_eq!(c.get_cell_at(&vec![1,2,3]).unwrap().unwrap(), checkcell);

		checkcell.display_content = "hl".to_string();
		let dupe = checkcell.clone();

		assert!(c.set_cell_at(&vec![1,2,3], checkcell).is_ok());
		assert_eq!(c.get_cell_at(&vec![1,2,3]).unwrap().unwrap(), dupe);
	}

	#[test]
	fn wrong_coords_set() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		let dupe = cell.clone();
		assert!(c.insert(&vec![1,2,3], cell).is_ok());
		assert!(c.set_cell_at(&vec![1,2], dupe).is_err());
	}

	#[test]
	fn removing() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		assert!(c.insert(&vec![1,2,3], cell).is_ok());
		assert!(c.remove(&vec![1,2,3]).is_ok());
		assert!(c.get_cell_at(&vec![1,2,3]).unwrap().is_none());
	}

	#[test]
	fn wrong_coords_remove() {
		let c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		let dupe = cell.clone();
		assert!(c.insert(&vec![1,2,3], cell).is_ok());
		assert!(c.remove(&vec![1,2]).is_err());
		assert_eq!(c.get_cell_at(&vec![1,2,3]).unwrap().unwrap(), dupe);
	}

	#[test]
	fn new_axes() {
		let mut c = MapContainer::new(3);

		assert!(c.set_axes(1).is_ok());
		assert_eq!(c.get_axes(), 1);
	}

	#[test]
	fn red_axes_content() {
		let mut c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		let dupe = cell.clone();
		assert!(c.insert(&vec![1,0,0], cell).is_ok());

		assert!(c.set_axes(1).is_ok());
		assert_eq!(c.get_axes(), 1);

		assert_eq!(c.get_cell_at(&vec![1]).unwrap().unwrap(), dupe);
	}

	#[test]
	fn red_axes_retain() {
		let mut c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		let dupe = cell.clone();
		assert!(c.insert(&vec![1,1,0], cell).is_ok());

		assert!(c.set_axes(2).is_ok());
		assert_eq!(c.get_axes(), 2);
		assert!(c.remove_axis(1, Some(1)).is_ok());

		assert_eq!(c.get_cell_at(&vec![1]).unwrap().unwrap(), dupe);
	}

	#[test]
	fn more_axes_content() {
		let mut c = MapContainer::new(3);
		let mut cell = Cell::default();
		cell.properties.bgcolor = Color::from_tuple((2,3,4));
		let dupe = cell.clone();
		assert!(c.insert(&vec![1,0,0], cell).is_ok());

		assert!(c.set_axes(7).is_ok());
		assert_eq!(c.get_axes(), 7);

		assert_eq!(c.get_cell_at(&vec![1,0,0,0,0,0,0]).unwrap().unwrap(), dupe);
	}

	#[test]
	fn multaxes() {
		let c = MapContainer::new(3);
		for i in 0..10 {
			for j in 0..10 {
				for k in 0..10 {
					let mut cell = Cell::default();
					cell.properties.bgcolor = Color::from_tuple((i*10,j*10,k*10));
					assert!(c.insert(&vec![i.into(), j.into(), k.into()], cell).is_ok());
				}
			}
		}

		let results = c.get_cells_at_axis(&[AnyPosition::Any, AnyPosition::Position(3), AnyPosition::Position(3)]).unwrap();

		for r in results {
			assert!(r.properties.bgcolor.r < 101);
			assert!(r.properties.bgcolor.g == 30);
			assert!(r.properties.bgcolor.b == 30);
		}
	}

	#[test]
	fn multaxes2() {
		let c = MapContainer::new(3);
		for i in 0..10 {
			for j in 0..10 {
				for k in 0..10 {
					let mut cell = Cell::default();
					cell.properties.bgcolor = Color::from_tuple((i*10,j*10,k*10));
					assert!(c.insert(&vec![i.into(), j.into(), k.into()], cell).is_ok());
				}
			}
		}

		let results = c.get_cells_at_axis(&[AnyPosition::Any, AnyPosition::Any, AnyPosition::Position(3)]).unwrap();

		for r in results {
			assert!(r.properties.bgcolor.r < 101);
			assert!(r.properties.bgcolor.g < 101);
			assert!(r.properties.bgcolor.b == 30);
		}
	}
}