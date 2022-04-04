use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]
pub struct AmountsUpdated {
	/// Available ore in player's purse
	#[property]
	pub total_ore: i32,

	/// Map of instance ID (ore structures) to remaining amount in that field
	/// Type: \[int] -> int
	#[property]
	pub ore_fields_remaining_amounts: Dictionary,

	/// Where +/- ore happened
	#[property]
	pub animated_positions: Vector2Array,

	/// How much +/- ore (same size as above)
	#[property]
	pub animated_diffs: Int32Array,
}

#[methods]
impl AmountsUpdated {
	fn new(_base: &Reference) -> Self {
		Self::default()
	}
}
