use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]
#[inherit(Spatial)]
pub struct Spatials {
	#[property]
	pub unimplemented: i32,
}

#[methods]
impl Spatials {
	fn new(_base: &Spatial) -> Self {
		godot_print!("Spatials is instantiated.");
		Self::default()
	}
}
