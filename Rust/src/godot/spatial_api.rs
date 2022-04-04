use gdnative::prelude::*;
use rand::prelude::*;
use rstar::{RTree, AABB};
use std::collections::{HashMap, HashSet};
//use std::collections::HashMap;

use crate::godot::{AddStructure, AmountsUpdated, BlightUpdateResult, QueryResult, Terrain};
use crate::objects::{Pipe, Structure, StructureType};
use crate::{Vector2Ext, Vector3Ext};

const DAMAGE_PER_SECOND: f32 = 80.0;
const STRUCTURE_HEALTH: f32 = 100.0;

/// The amount of collected ore per simulation tick when there's an active miner
const ORE_PER_COLLECTION: i32 = 5;
/// The frequency, in number of physics frames, after which active miners will
/// collect ore
const MINER_TICK_FREQ: usize = 60 * 2;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct SpatialApi {
	rtree: RTree<Structure>,
	structures_by_id: HashMap<i64, Structure>,
	pipes: Vec<Pipe>,

	terrain: Option<Instance<Terrain>>,

	scenes: Dictionary,
	ore_amount: i32,
	frame_count: usize,
}

#[methods]
impl SpatialApi {
	fn new(_base: &Spatial) -> Self {
		godot_print!("Spatials is instantiated.");

		Self {
			rtree: RTree::new(),
			structures_by_id: HashMap::new(),
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
			self.structures_by_id.insert(stc.instance_id(), stc);
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

	// Separate function to avoid reorder bugs in GDScript
	#[export]
	fn update_frame_count(&mut self, _base: &Spatial) {
		self.frame_count += 1;
	}

	#[export]
	fn update_blight(&mut self, base: &Spatial, dt: f32) -> Instance<BlightUpdateResult> {
		let result = if let Some(inst) = self.terrain.as_mut() {
			inst.map_mut(|terrain, _| {
				Self::update_blight_impl(
					&mut self.rtree,
					&mut self.pipes,
					&mut self.structures_by_id,
					dt,
					terrain,
				)
			})
			.unwrap()
		} else {
			BlightUpdateResult::default()
		};

		if !result.removed_pipe_ids.is_empty() {
			let world = base.get_parent().unwrap();
			self.update_pipe_network(world);
		}

		Instance::emplace(result).into_shared()
	}

	/// Returns the amount of ore collected
	#[profiling::function]
	#[allow(unreachable_code)]
	fn update_blight_impl(
		rtree: &mut RTree<Structure>,
		pipes: &mut Vec<Pipe>,
		structures_by_id: &mut HashMap<i64, Structure>,
		dt: f32,
		terrain: &mut Terrain,
	) -> BlightUpdateResult {
		let mut structures_to_remove = vec![];

		for stc in rtree.iter_mut() {
			profiling::scope!("blight");

			if stc.is_powered() {
				if let Some(radius) = stc.clean_radius() {
					terrain.clean_circle(stc.position().to_3d(), radius);
				}
			}

			if let Some(damage_radius) = stc.damage_radius() {
				let blight =
					terrain.get_average_blight_in_circle(stc.position().to_3d(), damage_radius);

				let damage = dt * DAMAGE_PER_SECOND * blight as f32 / 256.0;
				stc.deal_damage(damage);
			}

			if !stc.is_alive() {
				structures_to_remove.push(*stc);
			}
		}

		// Remove destroyed structures
		let mut removed_pipe_ids = vec![];
		for elem in structures_to_remove.iter() {
			rtree.remove(elem);
			structures_by_id.remove(&elem.instance_id());

			let node_id = elem.instance_id();
			let node = unsafe { Node::from_instance_id(node_id) };
			node.queue_free();

			// O(n^2), could be done by Multimap<i64, Structure> (or HashMap<i64, Vec<Structure>>) but likely won't matter in this scale
			//pipes.retain(|pipe| pipe.start_id() != node_id && pipe.end_id() != node_id);

			let mut i = 0;
			while i < pipes.len() {
				let pipe = pipes[i];
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

		BlightUpdateResult { removed_pipe_ids }
	}

	#[export]
	fn update_amounts(&mut self, _base: &Spatial) -> Option<Instance<AmountsUpdated>> {
		if self.frame_count % MINER_TICK_FREQ != 0 {
			return None;
		}

		let mut animated_positions = Vector2Array::new();
		let mut animated_diffs = Int32Array::new();

		// Iterate irrigators
		for irrigator in self.structures_by_id.values() {
			// Skip non-irrigators and inactive ones
			if irrigator.ty() != StructureType::Irrigation || !irrigator.is_powered() {
				continue;
			}

			let surrounding = Self::iter_structures_in_radius(
				&self.rtree,
				irrigator.position(),
				irrigator.clean_radius().unwrap(),
			);

			for stc in surrounding {
				if stc.ty() == StructureType::Ore {
					self.ore_amount += ORE_PER_COLLECTION;
					animated_positions.push(stc.position());
					animated_diffs.push(-ORE_PER_COLLECTION);
				}
			}
		}

		let result = AmountsUpdated {
			total_ore: self.ore_amount,
			animated_positions,
			animated_diffs,
		};

		Some(Instance::emplace(result).into_shared())
	}

	fn iter_structures_in_radius(
		rtree: &RTree<Structure>,
		position: Vector2,
		radius: f32,
	) -> impl Iterator<Item = Structure> + '_ {
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
	fn query_effect_radius(&self, _base: &Spatial, node: Ref<Spatial>) -> Instance<QueryResult> {
		// TODO O(n), could be HashMap'ed
		let stc = self.structures_by_id.get(&node.get_instance_id());
		let stc = stc.expect("Queried non-structure object, make sure that collision shapes of other objects are disabled");

		// Drawing a circle of radius 0 is OK if there's nothing
		let radius = stc.clean_radius().unwrap_or(0.0);
		let affected_ids = self.query_affected_ids(node.translation(), radius);

		let result = QueryResult {
			radius,
			affected_ids,
		};
		Instance::emplace(result).into_shared()
	}

	fn query_affected_ids(&self, position3d: Vector3, radius: f32) -> Vec<i64> {
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

	fn update_pipe_network(&mut self, world: Ref<Node>) {
		// Note: of course, we could only update from the changed pipes, but this is a more general approach
		println!("Update pipe network...");

		// First, find all water spots connected to pipes
		let mut visited_structures = HashSet::<i64>::new();
		let mut powered_structures = HashSet::<i64>::new();
		let mut powered_pipes = HashSet::<i64>::new();
		let mut graph_roots = Vec::new();

		for pipe in self.pipes.iter() {
			let pipe_id = pipe.pipe_node_id();

			let start_id = pipe.start_node_id();
			let start_stc = *self.structures_by_id.get(&start_id).unwrap();
			if start_stc.ty() == StructureType::Water {
				powered_structures.insert(start_id);
				powered_pipes.insert(pipe_id);
				graph_roots.push((pipe_id, start_id));
			}

			let end_id = pipe.end_node_id();
			let end_stc = *self.structures_by_id.get(&end_id).unwrap();
			if end_stc.ty() == StructureType::Water {
				powered_structures.insert(end_id);
				powered_pipes.insert(pipe_id);
				graph_roots.push((pipe_id, end_id));
			}
		}

		for (pipe_id, stc_id) in graph_roots {
			Self::recurse_pipe_graph(
				pipe_id,
				stc_id,
				&self.pipes,
				&self.structures_by_id,
				&mut powered_structures,
				&mut powered_pipes,
				&mut visited_structures,
			);
		}

		// Apply changes to every structure
		for stc in self.rtree.iter_mut() {
			let id = stc.instance_id();
			let powered = powered_structures.contains(&id);

			if !stc.can_be_powered() {
				continue;
			}

			// Update in 2 places (keep map and rtree in sync)
			stc.set_powered(powered);
			Self::sync_structure(*stc, &mut self.structures_by_id);
		}

		for pipe in self.pipes.iter() {
			let id = pipe.pipe_node_id();
			let powered = powered_pipes.contains(&id);
			world.call("setPowered", &v![id, powered]);
		}

		//println!("Done updating pipe network.\n");
	}

	// Synchronize changes from RTRee to HashMap
	fn sync_structure(stc: Structure, structures_by_id: &mut HashMap<i64, Structure>) {
		let stc_mut = structures_by_id.get_mut(&stc.instance_id()).unwrap();
		*stc_mut = stc;
	}

	fn recurse_pipe_graph(
		pipe_id: i64,
		stc_id: i64,
		pipes: &Vec<Pipe>,
		structures_by_id: &HashMap<i64, Structure>,
		powered_structures: &mut HashSet<i64>,
		powered_pipes: &mut HashSet<i64>,
		visited_structures: &mut HashSet<i64>,
	) {
		if visited_structures.contains(&stc_id) {
			return;
		}
		visited_structures.insert(stc_id);

		let stc = structures_by_id.get(&stc_id).unwrap();
		if stc.can_be_powered() {
			//println!("    Power {stc_id}!");
			powered_pipes.insert(pipe_id);
			powered_structures.insert(stc_id);
		}

		for (pipe_id, adjacent_id) in
			Self::get_pipe_adjacent_pairs(stc_id, pipes, visited_structures)
		{
			//println!("  Explore pipe {pipe_id}: connects {stc_id} -> {adjacent_id}...");
			Self::recurse_pipe_graph(
				pipe_id,
				adjacent_id,
				pipes,
				structures_by_id,
				powered_structures,
				powered_pipes,
				visited_structures,
			);
		}
	}

	fn get_pipe_adjacent_pairs(
		node_id: i64,
		pipes: &Vec<Pipe>,
		except: &HashSet<i64>,
	) -> Vec<(i64, i64)> {
		// Slow, whatever
		let mut result = vec![];
		for pipe in pipes {
			let pipe_id = pipe.pipe_node_id();
			let start_id = pipe.start_node_id();
			let end_id = pipe.end_node_id();

			if start_id == node_id {
				if !except.contains(&end_id) {
					result.push((pipe_id, end_id));
				}
			} else if end_id == node_id {
				if !except.contains(&start_id) {
					result.push((pipe_id, start_id));
				}
			}
		}

		result
	}

	#[export]
	fn add_structure(&mut self, base: &Spatial, added: Instance<AddStructure>) -> i64 {
		let added: AddStructure = added.map(|inst, _| inst.clone()).unwrap();

		let stc = self.instance_structure(base, added.position.to_2d(), &added.structure_ty);
		godot_print!("Add structure {:?}", stc);

		if let Some(from) = added.pipe_from_obj {
			let pipe_id = self.instance_pipe(base, from.translation(), added.position);
			let from_id = from.get_instance_id();
			let stc_id = stc.instance_id();
			let to_id = stc_id;

			self.pipes.push(Pipe::new(pipe_id, from_id, to_id));
		}

		self.structures_by_id.insert(stc.instance_id(), stc);
		self.rtree.insert(stc);

		let world = base.get_parent().unwrap();
		self.update_pipe_network(world);

		stc.instance_id()
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
