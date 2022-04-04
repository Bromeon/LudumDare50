use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default, Clone)]
pub struct AddStructure {
	#[property]
	pub position: Vector3,

	#[property]
	pub structure_ty: String,

	#[property]
	pub pipe_from_obj: Option<Ref<Spatial>>,
}

#[methods]
impl AddStructure {
	fn new(_base: &Reference) -> Self {
		Self::default()
	}
}
