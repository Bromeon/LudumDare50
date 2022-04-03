use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]
#[inherit(Spatial)]
pub struct Zeppelin {
	#[property]
	pub unimplemented: i32,
}

#[methods]
impl Zeppelin {
	fn new(_base: &Spatial) -> Self {
		Self::default()
	}
}
