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
	amount: Option<u32>,
}

impl Structure {
	pub fn new(ty: StructureType, position: Vector2, id: i64, health: f32) -> Structure {
		Self {
			ty,
			position,
			id,
			health,
			powered: Self::initially_powered(ty),
			amount: Self::initial_amount(ty),
		}
	}

	// The radius used when checking if this building is taking damage from blight
	pub fn damage_radius(&self) -> Option<f32> {
		match self.ty {
			StructureType::Water => None, // Doesn't take damage
			StructureType::Ore => None,   // Doesn't take damage
			StructureType::Pump => Some(1.0),
			StructureType::Irrigation => Some(1.5),
		}
	}

	// When this building is powered, the
	pub fn clean_radius(&self) -> Option<f32> {
		match self.ty {
			StructureType::Water => Some(3.0),
			StructureType::Ore => None, // Doesn't clean
			StructureType::Pump => None,
			StructureType::Irrigation => Some(8.0),
		}
	}

	// Setters
	pub fn deal_damage(&mut self, damage: f32) {
		assert!(self.damage_radius().is_some());
		self.health -= damage;
	}

	pub fn set_powered(&mut self, powered: bool) {
		assert!(self.can_be_powered());
		self.powered = powered;
	}

	// Getters
	pub fn ty(&self) -> StructureType {
		self.ty
	}

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

	pub fn can_be_powered(&self) -> bool {
		match self.ty {
			StructureType::Water => false, // not toggleable
			StructureType::Ore => false,
			StructureType::Pump => true,
			StructureType::Irrigation => true,
		}
	}

	pub fn amount(&self) -> u32 {
		self.amount.expect("Queried amount of invalid type")
	}

	// Constructor helpers
	fn initially_powered(ty: StructureType) -> bool {
		match ty {
			StructureType::Water => true,
			StructureType::Ore => false,
			StructureType::Pump => false,
			StructureType::Irrigation => false,
		}
	}

	fn initial_amount(ty: StructureType) -> Option<u32> {
		match ty {
			StructureType::Water => Some(50),
			StructureType::Ore => Some(50),
			StructureType::Pump => None,
			StructureType::Irrigation => None,
		}
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
