// Auto-generated; do not edit.

mod add_structure;
mod amounts_updated;
mod blight_updated;
mod query_result;
mod spatial_api;
mod terrain;
mod zeppelin;

pub use add_structure::*;
pub use amounts_updated::*;
pub use blight_updated::*;
pub use query_result::*;
pub use spatial_api::*;
pub use terrain::*;
pub use zeppelin::*;

pub fn register_classes(handle: gdnative::init::InitHandle) {
	handle.add_class::<SpatialApi>();
	handle.add_class::<Terrain>();
	handle.add_class::<Zeppelin>();
	handle.add_class::<AddStructure>();
	handle.add_class::<BlightUpdated>();
	handle.add_class::<AmountsUpdated>();
	handle.add_class::<QueryResult>();
}
