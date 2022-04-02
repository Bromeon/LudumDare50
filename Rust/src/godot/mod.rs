// Auto-generated; do not edit.

mod godot_api;
mod little_struct;

pub use godot_api::*;
pub use little_struct::*;

pub fn register_classes(handle: gdnative::init::InitHandle) {
	handle.add_class::<GodotApi>();
	handle.add_class::<LittleStruct>();
}

