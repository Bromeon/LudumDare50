// Auto-generated; do not edit.

mod spatial_api;
mod terrain;
mod zeppelin;
mod add_structure;
mod blight_update_result;

pub use spatial_api::*;
pub use terrain::*;
pub use zeppelin::*;
pub use add_structure::*;
pub use blight_update_result::*;

pub fn register_classes(handle: gdnative::init::InitHandle) {
	handle.add_class::<SpatialApi>();
	handle.add_class::<Terrain>();
	handle.add_class::<Zeppelin>();
	handle.add_class::<AddStructure>();
	handle.add_class::<BlightUpdateResult>();
}
