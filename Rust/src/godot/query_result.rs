use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]
pub struct QueryResult {
	#[property(get = "Self::get_affected_ids")]
	pub affected_ids: Vec<i64>,

	#[property]
	pub radius: f32,
}

#[methods]
impl QueryResult {
	fn new(_base: &Reference) -> Self {
		Self::default()
	}

	fn get_affected_ids(&self, _base: TRef<Reference>) -> VariantArray {
		VariantArray::from_iter(self.affected_ids.iter()).into_shared()
	}
}
