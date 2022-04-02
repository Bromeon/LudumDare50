// Auto-generated; do not edit.

mod godot_api;
mod little_struct;
mod spatials;
mod terrain;

pub use godot_api::*;
pub use little_struct::*;
pub use spatials::*;
pub use terrain::*;

pub fn register_classes(handle: gdnative::init::InitHandle) {
	handle.add_class::<GodotApi>();
	handle.add_class::<LittleStruct>();
	handle.add_class::<Terrain>();
	handle.add_class::<Spatials>();
}
