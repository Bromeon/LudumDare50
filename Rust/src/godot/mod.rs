// Auto-generated; do not edit.

mod spatial_api;
mod terrain;
mod zeppelin;

pub use spatial_api::*;
pub use terrain::*;
pub use zeppelin::*;

pub fn register_classes(handle: gdnative::init::InitHandle) {
	handle.add_class::<SpatialApi>();
	handle.add_class::<Terrain>();
	handle.add_class::<Zeppelin>();
}
