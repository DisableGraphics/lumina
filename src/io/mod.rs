use std::{error::Error, fs::File, io::Read, path::Path};

use crate::container::{Container, mapcontainer::MapContainer};

/// ContainerBuilder makes a Container that can be used by application code
pub struct ContainerBuilder {

}

impl ContainerBuilder {
	/// Creates a new ContainerBuilder
	pub fn new() -> Self {
		Self {}
	}

	/// Creates a container from a file
	pub fn file<P: AsRef<Path>>(self, path: P) -> Result<Box<dyn Container>, Box<dyn Error>> {
		let mut f = File::open(path)?;
		let mut vec = Vec::new();
		f.read_to_end(&mut vec)?;
		let c: MapContainer = postcard::from_bytes(&vec)?;
		Ok(Box::new(c))
	}

	/// Creates a container from a reader over binary data
	pub fn reader(self, reader: &mut dyn Read) -> Result<Box<dyn Container>, Box<dyn Error>> {
		let mut vec = Vec::new();
		reader.read_to_end(&mut vec)?;
		let c: MapContainer = postcard::from_bytes(&vec)?;
		Ok(Box::new(c))
	}
	/// Creates an empty container
	pub fn empty(self, axes: usize) -> Result<Box<dyn Container>, Box<dyn Error>> {
		let c: MapContainer = MapContainer::new(axes);
		Ok(Box::new(c))
	}
	#[allow(dead_code)]
	fn file_inner<P: AsRef<Path>>(self, path: P) -> Result<MapContainer, Box<dyn Error>> {
		let mut f = File::open(path)?;
		let mut vec = Vec::new();
		f.read_to_end(&mut vec)?;
		let c: MapContainer = postcard::from_bytes(&vec)?;
		Ok(c)
	}
	#[allow(dead_code)]
	fn empty_inner(self, axes: usize) -> Result<MapContainer, Box<dyn Error>> {
		let c: MapContainer = MapContainer::new(axes);
		Ok(c)
	}
}

impl Default for ContainerBuilder {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod test {
    use std::{error::Error, fs::File, io::Write, path::Path};

	use postcard::to_allocvec;

	use crate::{Cell, container::Container, io::ContainerBuilder};
	fn create_if_not_available() -> Result<(), Box<dyn Error>> {
		let p = Path::new("tests/data/src.lum");
		if !p.exists() {
			let coso = ContainerBuilder::new().empty_inner(3)?;
			let v = to_allocvec(&coso)?;
			let mut f = File::create(p)?;
			f.write_all(&v)?;
		}
		Ok(())
	}

	#[test]
	fn load() {
		create_if_not_available().unwrap();
		assert!(ContainerBuilder::new().file("tests/data/src.lum").is_ok());
	}

	fn create_if_not_available2() -> Result<(), Box<dyn Error>> {
		let p = Path::new("tests/data/src2.lum");
		if !p.exists() {
			let coso = ContainerBuilder::new().empty_inner(3)?;
			coso.insert(&vec![1,2,3], Cell::default())?;
			let v = to_allocvec(&coso)?;
			let mut f = File::create(p)?;
			f.write_all(&v)?;
		}
		Ok(())
	}

	#[test]
	fn load2() {
		create_if_not_available2().unwrap();
		let p = ContainerBuilder::new().file("tests/data/src2.lum").unwrap();
		assert_eq!(p.get_cell_at(&vec![1,2,3]).unwrap().unwrap(), Cell::default())
	}

	fn create_if_not_available3() -> Result<(), Box<dyn Error>> {
		let p = Path::new("tests/data/src3.lum");
		if !p.exists() {
			let coso: crate::container::mapcontainer::MapContainer = ContainerBuilder::new().empty_inner(3)?;
			for i in 0..100 {
				for j in 0..100 {
					for k in 0..100 {
						coso.insert(&vec![i,j,k], Cell::default())?;
					}
				}
			}
			let v = to_allocvec(&coso)?;
			let mut f = File::create(p)?;
			f.write_all(&v)?;
		}
		Ok(())
	}

	#[test]
	fn load3() {
		create_if_not_available3().unwrap();
		let p = ContainerBuilder::new().file("tests/data/src3.lum").unwrap();
		assert_eq!(p.get_n_cells(), 100*100*100);
	}
}