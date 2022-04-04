// Auto-generated; do not edit.

mod spatial_api;
mod terrain;
mod zeppelin;
mod add_structure;
mod blight_updated;
mod amounts_updated;
mod query_result;

pub use spatial_api::*;
pub use terrain::*;
pub use zeppelin::*;
pub use add_structure::*;
pub use blight_updated::*;
pub use amounts_updated::*;
pub use query_result::*;

pub fn register_classes(handle: gdnative::init::InitHandle) {
	handle.add_class::<SpatialApi>();
	handle.add_class::<Terrain>();
	handle.add_class::<Zeppelin>();
	handle.add_class::<AddStructure>();
	handle.add_class::<BlightUpdated>();
	handle.add_class::<AmountsUpdated>();
	handle.add_class::<QueryResult>();
}
