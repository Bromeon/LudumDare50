use gdnative::prelude::*;
use rand::prelude::*;
use rstar::{RTree, AABB};
//use std::collections::HashMap;

use crate::godot::Terrain;
use crate::objects::{Structure, StructureType, IRRIGATION_CLEAN_RADIUS, Pipe};
use crate::{Vector2Ext, Vector3Ext};

const DAMAGE_PER_SECOND: f32 = 80.0;
const STRUCTURE_HEALTH: f32 = 100.0;

/// The amount of collected ore per simulation tick when there's an active miner
const ORE_PER_COLLECTION: u32 = 5;
/// The frequency, in number of physics frames, after which active miners will
/// collect ore
const MINER_TICK_FREQ: usize = 60 * 2;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct SpatialApi {
	//structures_by_id: HashMap<i64, Structure>,
	rtree: RTree<Structure>,
	pipes: Vec<Pipe>,

	terrain: Option<Instance<Terrain>>,

	stc_scenes: Dictionary,
	ore_amount: u32,
	frame_count: usize,
}

#[methods]
impl SpatialApi {
	fn new(_base: &Spatial) -> Self {
		godot_print!("Spatials is instantiated.");

		Self {
			//structures_by_id: HashMap::new(),
			rtree: RTree::new(),
			pipes: Vec::new(),
			terrain: None,
			stc_scenes: Dictionary::new_shared(),
			ore_amount: 0,
			frame_count: 0,
		}
	}

	#[export]
	fn load(&mut self, base: &Spatial, scenes: Dictionary) {
		self.stc_scenes = scenes;

		let mut structures = vec![];

		let variants = ["Water", "Ore", "Pump", "Irrigation"];
		for pos in random_positions(20) {
			let ty_name = variants.into_iter().choose(&mut thread_rng()).unwrap();
			let stc = self.instance_structure(base, pos, ty_name);

			structures.push(stc);
		}

		godot_print!("Bulk-add {} structures", structures.len());
		self.rtree = RTree::bulk_load(structures);
		self.terrain = Some(base.get_node_as_instance::<Terrain>("../Terrain").claim());
	}

	fn instance_structure(&self, base: &Spatial, pos: Vector2, ty_name: &str) -> Structure {
		let scene = self
			.stc_scenes
			.get(ty_name)
			.unwrap()
			.to_object::<PackedScene>()
			.unwrap();
		let instanced = scene.instance(0).unwrap();
		let instanced = instanced.cast::<Spatial>();
		let id = instanced.get_instance_id();

		instanced.set_translation(pos.to_3d());
		instanced.set_scale(0.2 * Vector3::ONE);
		base.get_node("Structures")
			.unwrap()
			.add_child(instanced, false);

		let ty = match ty_name {
			"Water" => StructureType::Water,
			"Ore" => StructureType::Ore,
			"Pump" => StructureType::Pump,
			"Irrigation" => StructureType::Irrigation,
			_ => unreachable!(),
		};

		Structure::new(ty, pos, id, STRUCTURE_HEALTH)
	}

	#[export]
	fn update_blight_impact(&mut self, _base: &Spatial, dt: f32) {
		self.frame_count += 1;
		if let Some(inst) = self.terrain.as_mut() {
			let collected_ore = inst
				.map_mut(|terrain, _| {
					Self::update_blight_impl(&mut self.rtree, dt, terrain, self.frame_count)
				})
				.unwrap();
			self.ore_amount += collected_ore;
		}
	}

	/// Returns the amount of ore collected
	#[profiling::function]
	#[allow(unreachable_code)]
	fn update_blight_impl(
		rtree: &mut RTree<Structure>,
		dt: f32,
		terrain: &mut Terrain,
		frame_count: usize,
	) -> u32 {
		let mut to_remove = vec![];
		let mut ores = vec![];

		for stc in rtree.iter_mut() {
			profiling::scope!("blight");

			if stc.is_powered() {
				terrain.clean_circle(stc.position().to_3d(), stc.clean_radius());
			} else if let Some(damage_radius) = stc.damage_radius() {
				let blight =
					terrain.get_average_blight_in_circle(stc.position().to_3d(), damage_radius);

				let damage = dt * DAMAGE_PER_SECOND * blight as f32 / 256.0;
				stc.deal_damage(damage);
			}

			if frame_count % MINER_TICK_FREQ == 0 && matches!(stc.ty(), StructureType::Ore) {
				ores.push(*stc);
			}

			if !stc.is_alive() {
				to_remove.push(*stc);
			}
		}

		// RTree API only allows removal one at a time
		if !to_remove.is_empty() {
			//println!("Remove {} structures", to_remove.len());
		}

		let mut ore_collected = 0;
		if frame_count % MINER_TICK_FREQ == 0 {
			for ore in ores {
				// If there's at least one irrigation covering this ore, then it
				// will collect ore for the frame.
				if Self::iter_structures_in_radius(rtree, ore.position(), IRRIGATION_CLEAN_RADIUS)
					.any(|other| matches!(other.ty(), StructureType::Ore))
				{
					ore_collected += ORE_PER_COLLECTION;
				}
			}
		}

		for elem in to_remove.iter() {
			rtree.remove(elem);

			let node = unsafe { Node::from_instance_id(elem.instance_id()) };
			node.queue_free();
		}

		ore_collected
	}

	fn iter_structures_in_radius(
		rtree: &RTree<Structure>,
		position: Vector2,
		radius: f32,
	) -> impl Iterator<Item = Structure> + '_ {
		//self.structures_by_id.keys().copied().collect()
		let half_size = Vector2::ONE * radius;
		let center = position;
		let p1 = (center - half_size).to_rstar();
		let p2 = (center + half_size).to_rstar();

		let aabb = AABB::from_corners(p1, p2);
		//println!("Query {:?}", aabb);

		let radius_sq = radius * radius;
		rtree
			.locate_in_envelope(&aabb)
			.filter(move |stc| stc.position().distance_squared_to(center) < radius_sq)
			.copied()
	}

	#[export]
	fn query_radius(&self, _base: &Spatial, position3d: Vector3, radius: f32) -> Vec<i64> {
		//self.structures_by_id.keys().copied().collect()
		let half_size = Vector2::ONE * radius;
		let center = position3d.to_2d();
		let p1 = (center - half_size).to_rstar();
		let p2 = (center + half_size).to_rstar();

		let aabb = AABB::from_corners(p1, p2);
		//println!("Query {:?}", aabb);

		let radius_sq = radius * radius;
		self.rtree
			.locate_in_envelope(&aabb)
			.filter(|stc| stc.position().distance_squared_to(center) < radius_sq)
			.map(|stc| stc.instance_id())
			.collect()
	}

	#[export]
	fn add_structure(&mut self, base: &Spatial, pos: Vector3, ty_name: String, pipe_from: Option<Ref<Spatial>>) -> i64 {
		let stc = self.instance_structure(base, pos.to_2d(), &ty_name);
		godot_print!("Add structure {:?}", stc);

		if let Some(node) = pipe_from {
			let from_id = node.get_instance_id();
			let to_id = stc.instance_id();

			self.pipes.push(Pipe::new(from_id, to_id));
		}

		//self.structures_by_id.insert(id, stc);
		self.rtree.insert(stc);
		stc.instance_id()
	}

	#[export]
	fn get_ore_amount(&mut self, _base: &Spatial) -> u32 {
		self.ore_amount
	}
}

fn random_positions(n: usize) -> Vec<Vector2> {
	let dist = rand::distributions::Uniform::new(-10.0, 10.0);

	//rand::thread_rng().g
	let mut result = vec![];
	for _i in 0..n {
		let x = rand::thread_rng().sample(dist);
		let y = rand::thread_rng().sample(dist);

		result.push(Vector2::new(x, y));
	}

	result
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
