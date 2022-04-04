use gdnative::prelude::*;
use rand::prelude::*;
use rstar::{RTree, AABB};
//use std::collections::HashMap;

use crate::godot::{AddStructure, BlightUpdateResult, Terrain};
use crate::objects::{Pipe, Structure, StructureType, IRRIGATION_CLEAN_RADIUS};
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

	scenes: Dictionary,
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
			scenes: Dictionary::new_shared(),
			ore_amount: 0,
			frame_count: 0,
		}
	}

	#[export]
	fn load(&mut self, base: &Spatial, scenes: Dictionary) {
		self.scenes = scenes;

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
		let (instanced, id) = self.instance_scene(ty_name);

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

	fn instance_pipe(&self, base: &Spatial, from: Vector3, to: Vector3) -> i64 {
		let (pipe, id) = self.instance_scene("Pipe");

		// Important: add to tree first!
		base.get_node("Pipes").unwrap().add_child(pipe, false);

		let world = base.get_parent().unwrap();
		world.call("alignPipe", &v![pipe, from, to]);
		id
	}

	fn instance_scene(&self, scene_key: &str) -> (Ref<Spatial>, i64) {
		let scene = self.scenes.get(scene_key).unwrap();
		let scene = scene.to_object::<PackedScene>().unwrap();
		let instanced = scene.instance(0).unwrap();
		let instanced = instanced.cast::<Spatial>();
		let id = instanced.get_instance_id();

		(instanced, id)
	}

	#[export]
	fn update_blight(&mut self, _base: &Spatial, dt: f32) -> Instance<BlightUpdateResult> {
		self.frame_count += 1;
		let result = if let Some(inst) = self.terrain.as_mut() {
			let result = inst
				.map_mut(|terrain, _| {
					Self::update_blight_impl(
						&mut self.rtree,
						&mut self.pipes,
						dt,
						terrain,
						self.frame_count,
					)
				})
				.unwrap();

			self.ore_amount += result.collected_ore;

			result
		} else {
			BlightUpdateResult::default()
		};

		Instance::emplace(result).into_shared()
	}

	/// Returns the amount of ore collected
	#[profiling::function]
	#[allow(unreachable_code)]
	fn update_blight_impl(
		rtree: &mut RTree<Structure>,
		pipes: &mut Vec<Pipe>,
		dt: f32,
		terrain: &mut Terrain,
		frame_count: usize,
	) -> BlightUpdateResult {
		let mut structures_to_remove = vec![];
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
				structures_to_remove.push(*stc);
			}
		}

		let mut collected_ore = 0;
		if frame_count % MINER_TICK_FREQ == 0 {
			for ore in ores {
				// If there's at least one irrigation covering this ore, then it
				// will collect ore for the frame.
				if Self::iter_structures_in_radius(rtree, ore.position(), IRRIGATION_CLEAN_RADIUS)
					.any(|other| matches!(other.ty(), StructureType::Ore))
				{
					collected_ore += ORE_PER_COLLECTION;
				}
			}
		}

		// Remove destroyed structures
		let mut removed_pipe_ids = vec![];
		for elem in structures_to_remove.iter() {
			rtree.remove(elem);

			let node_id = elem.instance_id();
			let node = unsafe { Node::from_instance_id(node_id) };
			node.queue_free();

			// O(n^2), could be done by Multimap<i64, Structure> (or HashMap<i64, Vec<Structure>>) but likely won't matter in this scale
			//pipes.retain(|pipe| pipe.start_id() != node_id && pipe.end_id() != node_id);

			let mut i = 0;
			while i < pipes.len() {
				let pipe = pipes[i].clone();
				if pipe.start_node_id() == node_id || pipe.end_node_id() == node_id {
					pipes.swap_remove(i);
					removed_pipe_ids.push(pipe.pipe_node_id());
				} else {
					i += 1;
				}
			}
		}

		if !removed_pipe_ids.is_empty() {
			godot_print!("Removed pipe IDs: {:?}", removed_pipe_ids);
		}

		BlightUpdateResult {
			collected_ore,
			removed_pipe_ids,
		}
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
	fn add_structure(&mut self, base: &Spatial, added: Instance<AddStructure>) -> i64 {
		let added: AddStructure = added.map(|inst, _| inst.clone()).unwrap();

		let stc = self.instance_structure(base, added.position.to_2d(), &added.structure_ty);
		godot_print!("Add structure {:?}", stc);

		if let Some(from) = added.pipe_from_obj {
			let pipe_id = self.instance_pipe(base, from.translation(), added.position);
			let from_id = from.get_instance_id();
			let to_id = stc.instance_id();

			self.pipes.push(Pipe::new(pipe_id, from_id, to_id));
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
