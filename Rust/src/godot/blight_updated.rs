use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]
pub struct BlightUpdated {
	#[property(get = "Self::get_removed_pipe_ids")]
	pub removed_pipe_ids: Vec<i64>,
}

#[methods]
impl BlightUpdated {
	fn new(_base: &Reference) -> Self {
		Self::default()
	}

	fn get_removed_pipe_ids(&self, _base: TRef<Reference>) -> VariantArray {
		VariantArray::from_iter(self.removed_pipe_ids.iter()).into_shared()
	}
}
