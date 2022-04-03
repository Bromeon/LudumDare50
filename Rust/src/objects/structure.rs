use gdnative::prelude::*;

use crate::Vector2Ext;
use rstar::{RTreeObject, AABB};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StructureType {
	Water,
	Ore,
	Pump,
	Irrigation,
}

#[derive(Debug, Copy, Clone)]
pub struct Structure {
	ty: StructureType,
	position: Vector2,
	id: i64,
	health: f32,
	powered: bool,
}

impl Structure {
	pub fn new(ty: StructureType, position: Vector2, id: i64, health: f32) -> Structure {
		Self {
			ty,
			position,
			id,
			health,
			powered: dbg!(match dbg!(ty) {
				StructureType::Water => true,
				StructureType::Ore => false,
				StructureType::Pump => false,
				StructureType::Irrigation => false,
			}),
		}
	}

	pub fn takes_damage(&self) -> bool {
		match self.ty {
			StructureType::Water => false,
			StructureType::Ore => false,
			StructureType::Pump => true,
			StructureType::Irrigation => true,
		}
	}

	// The radius used when checking if this building is taking damage from blight
	pub fn damage_radius(&self) -> f32 {
		match self.ty {
			StructureType::Water => 0.0, // Doesn't take damage
			StructureType::Ore => 0.0, // Doesn't take damage
			StructureType::Pump => 1.0,
			StructureType::Irrigation => 1.5,
		}
	}

	// When this building is powered, the 
	pub fn clean_radius(&self) -> f32 {
		match self.ty {
			StructureType::Water => 5.0,
			StructureType::Ore => 0.0, // Doesn't clean
			StructureType::Pump => 2.0,
			StructureType::Irrigation => 8.0,
		}
	}

	// Setters
	pub fn deal_damage(&mut self, damage: f32) {
		self.health -= damage;
	}
	pub fn set_powered(&mut self, powered: bool) {
		self.powered = powered;
	}

	// Getters
	pub fn instance_id(&self) -> i64 {
		self.id
	}
	pub fn position(&self) -> Vector2 {
		self.position
	}
	pub fn is_alive(&self) -> bool {
		self.health > 0.0
	}
	pub fn is_powered(&self) -> bool {
		self.powered
	}
}

impl RTreeObject for Structure {
	type Envelope = AABB<[f32; 2]>;

	fn envelope(&self) -> Self::Envelope {
		let aabb = AABB::from_point(self.position.to_rstar());
		aabb
	}
}

// for RTRee
impl PartialEq for Structure {
	fn eq(&self, other: &Self) -> bool {
		self.position == other.position
	}
}

impl Eq for Structure {}
