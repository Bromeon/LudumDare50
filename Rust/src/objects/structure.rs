use gdnative::prelude::*;

use crate::Vector2Ext;
use rstar::{RTreeObject, AABB};

#[derive(Debug, Copy, Clone)]
pub struct Structure {
	position: Vector2,
	id: i64,
	health: f32,
}

impl PartialEq for Structure {
	fn eq(&self, other: &Self) -> bool {
		self.position == other.position
	}
}

impl Eq for Structure{}

impl Structure {
	pub fn new(position: Vector2, id: i64, health:f32) -> Structure {
		Self {
			position,
			id,
			health,
		}
	}

	pub fn instance_id(&self) -> i64 {
		self.id
	}
	pub fn position(&self) -> Vector2 {
		self.position
	}

	pub fn deal_damage(&mut self, damage: f32) {
		self.health -= damage;
	}

	pub fn is_alive(&self) -> bool {
		self.health > 0.0
	}
}

impl RTreeObject for Structure {
	type Envelope = AABB<[f32; 2]>;

	fn envelope(&self) -> Self::Envelope {
		let aabb = AABB::from_point(self.position.to_rstar());
		aabb
	}
}
