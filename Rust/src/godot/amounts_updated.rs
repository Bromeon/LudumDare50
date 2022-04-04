use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]
pub struct AmountsUpdated {
	#[property]
	pub total_ore: i32,

	// Where +/- ore happened
	#[property]
	pub animated_positions: Vector2Array,

	// How much +/- ore (same size as above)
	#[property]
	pub animated_diffs: Int32Array,
}

#[methods]
impl AmountsUpdated {
	fn new(_base: &Reference) -> Self {
		Self::default()
	}
}
