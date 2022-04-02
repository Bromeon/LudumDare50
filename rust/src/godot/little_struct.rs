use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]
pub struct LittleStruct {
	#[property]
	pub unimplemented: i32,
}

#[methods]
impl LittleStruct {
	fn new(_base: &Reference) -> Self {
		Self::default()
	}
}
