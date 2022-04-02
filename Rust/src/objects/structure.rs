use gdnative::prelude::*;

use crate::VectorExt;
use rstar::{RTreeObject, AABB};

struct Structure {
	position: Vector2,
}

impl RTreeObject for Structure {
	type Envelope = AABB<[f32; 2]>;

	fn envelope(&self) -> Self::Envelope {
		let aabb = AABB::from_point(self.position.to_rstar());
		aabb
	}
}
