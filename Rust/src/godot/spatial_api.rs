use gdnative::prelude::*;
use rand::prelude::*;
use rstar::{RTree, AABB};
use std::collections::{HashMap, HashSet};
//use std::collections::HashMap;

use crate::godot::{AddStructure, AmountsUpdated, BlightUpdated, QueryResult, Terrain};
use crate::objects::{Pipe, Structure, StructureType};
use crate::{Vector2Ext, Vector3Ext};

const DAMAGE_PER_SECOND: f32 = 80.0;
const BLIGHT_THRESHOLD: u8 = 200;
const STRUCTURE_HEALTH: f32 = 100.0;

/// The amount of collected ore per simulation tick when there's an active miner
const ORE_PER_COLLECTION: i32 = 5;

/// The amount of water used by irrigators each second
const WATER_SPENT_PER_SECOND: i32 = 1;

/// The frequency, in number of physics frames, after which active miners will
/// collect ore
const MINER_TICK_FREQ: usize = 60 * 5;
const WATER_TICK_FREQ: usize = 60;

// Make sure water doesn't update at same time as ore (animations)
const WATER_TICK_OFFSET: usize = 30;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct SpatialApi {
	rtree: RTree<Structure>,
	structures_by_id: HashMap<i64, Structure>,
	/// Maps irrigators to their power source (Water structure)
	irrigators_by_powering_water: HashMap<i64, i64>,
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
			irrigators_by_powering_water: HashMap::new(),
			pipes: Vec::new(),
			terrain: None,
			scenes: Dictionary::new_shared(),
			ore_amount: 100,
			frame_count: 0,
		}
	}

	#[export]
	fn load(&mut self, base: &Spatial, scenes: Dictionary) {
		self.scenes = scenes;

		let mut structures = vec![];

		// Make sure those appear in any case in their specified amounts
		let mut at_least_available = vec!["Water", "Ore"];

		let variants = ["Water", "Ore", "Ore", "Ore"]; //, "Pump", "Irrigation"];
		for pos in random_positions(50) {
			let ty_name = if let Some(ty) = at_least_available.pop() {
				ty
			} else {
				variants.into_iter().choose(&mut thread_rng()).unwrap()
			};

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

		let ty = StructureType::from_name(ty_name);
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
	fn update_blight(&mut self, base: &Spatial, dt: f32) -> Instance<BlightUpdated> {
		let result = if let Some(inst) = self.terrain.as_mut() {
			inst.map_mut(|terrain, _| {
				Self::update_blight_impl(
					&mut self.rtree,
					&mut self.pipes,
					&mut self.structures_by_id,
					&mut self.irrigators_by_powering_water,
					dt,
					terrain,
				)
			})
			.unwrap()
		} else {
			BlightUpdated::default()
		};

		if !result.removed_pipe_ids.is_empty() {
			self.update_pipe_network(base);
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
		irrigators_by_powering_water: &mut HashMap<i64, i64>,
		dt: f32,
		terrain: &mut Terrain,
	) -> BlightUpdated {
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

				if blight > BLIGHT_THRESHOLD {
					let damage = dt * DAMAGE_PER_SECOND * blight as f32 / 256.0;
					stc.deal_damage(damage);
				}
			}

			if !stc.is_alive() {
				structures_to_remove.push(*stc);
			}
		}

		let removed_pipe_ids = Self::remove_structures_qualified(
			structures_to_remove,
			rtree,
			pipes,
			structures_by_id,
			irrigators_by_powering_water,
		);

		BlightUpdated { removed_pipe_ids }
	}

	/// Removes structures, updating refs
	fn remove_structures_qualified(
		structures_to_remove: Vec<Structure>,
		rtree: &mut RTree<Structure>,
		pipes: &mut Vec<Pipe>,
		structures_by_id: &mut HashMap<i64, Structure>,
		irrigators_by_powering_water: &mut HashMap<i64, i64>,
	) -> Vec<i64> {
		// Remove destroyed structures
		let mut removed_pipe_ids = vec![];
		for elem in structures_to_remove .iter() {
			let id_to_remove = elem.instance_id();

			unsafe {
				autoload::<Node>("Sfx")
					.unwrap()
					.call("stopMachineSound", &[elem.instance_id().to_variant()]);
			}
			rtree.remove(&elem);
			structures_by_id.remove(&id_to_remove);

			let node_id = id_to_remove;
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

			// Wipe all refs from this map too
			// Values removal is strictly not necessary (because water can't be removed), but this is sure to blow up in the future
			let mut keys_to_remove = vec![id_to_remove];
			for (key, value) in irrigators_by_powering_water.iter_mut() {
				if *value == id_to_remove {
					keys_to_remove.push(*key);
				}
			}
			for key in keys_to_remove {
				irrigators_by_powering_water.remove(&key);
			}
		}

		if !structures_to_remove.is_empty() {
			unsafe { autoload::<Node>("Sfx").unwrap().call("destroy", &[]) };
		}

		if !removed_pipe_ids.is_empty() {
			godot_print!("Removed pipe IDs: {:?}", removed_pipe_ids);
		}
		removed_pipe_ids
	}

	#[export]
	fn update_amounts(&mut self, base: &Spatial) -> Option<Instance<AmountsUpdated>> {
		let remaining_resource_amounts = Dictionary::new();
		let mut animated_positions = Vector2Array::new();
		let mut animated_diffs = Int32Array::new();
		let mut animated_strings = StringArray::new();

		let updated_water = self.update_water_amounts(
			&remaining_resource_amounts,
			&mut animated_positions,
			&mut animated_diffs,
			&mut animated_strings,
		);

		let updated_mines = self.update_mining_amounts(
			&remaining_resource_amounts,
			&mut animated_positions,
			&mut animated_diffs,
			&mut animated_strings,
		);

		let updated_water = match updated_water {
			WaterResult::NothingToDo => false,
			WaterResult::WaterConsumed => true,
			WaterResult::WaterDepleted => {
				self.update_pipe_network(base);
				true
			}
		};

		if updated_mines || updated_water {
			let result = AmountsUpdated {
				total_ore: self.ore_amount,
				remaining_resource_amounts: remaining_resource_amounts.into_shared(),
				animated_positions,
				animated_diffs,
				animated_strings,
			};

			Some(Instance::emplace(result).into_shared())
		} else {
			None
		}
	}

	fn update_water_amounts(
		&mut self,
		remaining_resource_amounts: &Dictionary<Unique>,
		animated_positions: &mut PoolArray<Vector2>,
		animated_diffs: &mut PoolArray<i32>,
		animated_strings: &mut PoolArray<GodotString>,
	) -> WaterResult {
		if (self.frame_count + WATER_TICK_OFFSET) % WATER_TICK_FREQ != 0 {
			return WaterResult::NothingToDo;
		}

		// Changed in RTree, needs HashMap update
		let mut structures_to_sync = vec![];

		let mut result = WaterResult::WaterConsumed;

		// Trace back each irrigator to its water source
		for irrigator in self
			.structures_by_id
			.values()
			.filter(|stc| Self::is_powered_irrigator(stc))
		{
			let water_id = self
				.irrigators_by_powering_water
				.get(&irrigator.instance_id())
				.expect("Irrigator powered without water source");

			// TODO again O(n^2)
			// the RTree should only store position + ID (they don't change)
			// all mutable attributes should be exclusively in the HashMap
			// let water_stc = self.structures_by_id.get_mut(water_id).unwrap();
			let mut water_in_rtree = None;
			for stc in self.rtree.iter_mut() {
				if stc.instance_id() == *water_id {
					water_in_rtree = Some(stc);
					break;
				}
			}
			let water_in_rtree = water_in_rtree.expect("water field not stored in RTree");

			let water_consumed = water_in_rtree.mine_amount(WATER_SPENT_PER_SECOND);
			structures_to_sync.push(*water_in_rtree);

			if water_consumed != 0 {
				animated_positions.push(water_in_rtree.position());
				animated_diffs.push(-water_consumed);
				animated_strings.push("Water".into());

				animated_positions.push(irrigator.position());
				animated_diffs.push(water_consumed);
				animated_strings.push("Water".into());
			} else {
				result = WaterResult::WaterDepleted;
			}
		}

		for water_in_rtree in structures_to_sync {
			Self::sync_structure(water_in_rtree, &mut self.structures_by_id);
		}

		// Iterate connected waters
		// TODO could also do changed only
		for water in self.structures_by_id.values() {
			if water.ty() != StructureType::Water {
				continue;
			}

			remaining_resource_amounts.insert(water.instance_id(), water.amount());
		}

		result
	}

	fn is_powered_irrigator(stc: &Structure) -> bool {
		stc.ty() == StructureType::Irrigation && stc.is_powered()
	}

	fn update_mining_amounts(
		&mut self,
		remaining_resource_amounts: &Dictionary<Unique>,
		animated_positions: &mut PoolArray<Vector2>,
		animated_diffs: &mut PoolArray<i32>,
		animated_strings: &mut PoolArray<GodotString>,
	) -> bool {
		if self.frame_count % MINER_TICK_FREQ != 0 {
			return false;
		}

		// Iterate irrigators (skip non-irrigators and inactive ones)

		for irrigator in self
			.structures_by_id
			.values()
			.filter(|stc| Self::is_powered_irrigator(stc))
		{
			let surrounding = Self::iter_structures_in_radius(
				&mut self.rtree,
				irrigator.position(),
				irrigator.clean_radius().unwrap(),
			);

			let mut mined_in_cycle = 0;
			for stc in surrounding {
				if stc.ty() == StructureType::Ore {
					let mined_amount = stc.mine_amount(ORE_PER_COLLECTION);
					mined_in_cycle += mined_amount;

					if mined_amount > 0 {
						animated_positions.push(stc.position());
						animated_diffs.push(-mined_amount);
						animated_strings.push("Ore".into());

						remaining_resource_amounts.insert(stc.instance_id(), stc.amount())
					}
				}
			}

			self.ore_amount += mined_in_cycle;
			if mined_in_cycle > 0 {
				animated_positions.push(irrigator.position());
				animated_diffs.push(mined_in_cycle);
				animated_strings.push("Ore".into());
			}
		}

		true
	}

	fn iter_structures_in_radius(
		rtree: &mut RTree<Structure>,
		position: Vector2,
		radius: f32,
	) -> impl Iterator<Item = &mut Structure> + '_ {
		let half_size = Vector2::ONE * radius;
		let center = position;
		let p1 = (center - half_size).to_rstar();
		let p2 = (center + half_size).to_rstar();

		let aabb = AABB::from_corners(p1, p2);
		//println!("Query {:?}", aabb);

		let radius_sq = radius * radius;
		rtree
			.locate_in_envelope_mut(&aabb)
			.filter(move |stc| stc.position().distance_squared_to(center) < radius_sq)
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

	fn update_pipe_network(&mut self, base: &Spatial) {
		// Note: of course, we could only update from the changed pipes, but this is a more general approach
		println!("Update pipe network...");

		// First, find all water spots connected to pipes
		let mut visited_structures = HashSet::<i64>::new();
		let mut powered_structures = HashMap::<i64, i64>::new(); // powered-stc mapped to original-water-id
		let mut powered_pipes = HashSet::<i64>::new();
		let mut graph_roots = Vec::new();

		for pipe in self.pipes.iter() {
			let pipe_id = pipe.pipe_node_id();

			let start_id = pipe.start_node_id();
			let start_stc = *self.structures_by_id.get(&start_id).unwrap();
			if start_stc.ty() == StructureType::Water && start_stc.amount() > 0 {
				powered_structures.insert(start_id, start_id); // water to itself
				powered_pipes.insert(pipe_id);
				graph_roots.push((pipe_id, start_id));
			}

			let end_id = pipe.end_node_id();
			let end_stc = *self.structures_by_id.get(&end_id).unwrap();
			if end_stc.ty() == StructureType::Water && start_stc.amount() > 0 {
				powered_structures.insert(end_id, end_id); // water to itself
				powered_pipes.insert(pipe_id);
				graph_roots.push((pipe_id, end_id));
			}
		}

		for (pipe_id, water_id) in graph_roots {
			Self::recurse_pipe_graph(
				pipe_id,
				water_id,
				water_id,
				&self.pipes,
				&self.structures_by_id,
				&mut powered_structures,
				&mut powered_pipes,
				&mut visited_structures,
			);
		}

		// Apply changes to every structure
		self.irrigators_by_powering_water.clear();

		let world = base.get_parent().unwrap();
		for stc in self.rtree.iter_mut() {
			let id = stc.instance_id();
			let powering_water = powered_structures.get(&id);

			if !stc.can_be_powered() {
				continue;
			}

			if let Some(water) = powering_water {
				self.irrigators_by_powering_water.insert(id, *water);
			}

			// Update in 2 places (keep map and rtree in sync)
			let powered = powering_water.is_some();
			stc.set_powered(powered);
			Self::sync_structure(*stc, &mut self.structures_by_id);
			world.call("setPowered", &v![stc.instance_id(), powered]);
		}

		for pipe in self.pipes.iter() {
			let id = pipe.pipe_node_id();
			let powered = powered_pipes.contains(&id);
			world.call("setPowered", &v![id, powered]);
		}

		//println!("Done updating pipe network.\n");
	}

	fn recurse_pipe_graph(
		pipe_id: i64,
		stc_id: i64,
		original_water_id: i64, // backtrack where water comes from
		pipes: &Vec<Pipe>,
		structures_by_id: &HashMap<i64, Structure>,
		powered_structures: &mut HashMap<i64, i64>,
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
			powered_structures.insert(stc_id, original_water_id);
		}

		for (pipe_id, adjacent_id) in
			Self::get_pipe_adjacent_pairs(stc_id, pipes, visited_structures)
		{
			//println!("  Explore pipe {pipe_id}: connects {stc_id} -> {adjacent_id}...");
			Self::recurse_pipe_graph(
				pipe_id,
				adjacent_id,
				original_water_id,
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

	// Synchronize changes from RTRee to HashMap
	fn sync_structure(stc: Structure, structures_by_id: &mut HashMap<i64, Structure>) {
		let stc_mut = structures_by_id.get_mut(&stc.instance_id()).unwrap();
		*stc_mut = stc;
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

		self.update_pipe_network(base);

		stc.instance_id()
	}

	#[export]
	fn can_consume_ore(&mut self, _base: &Spatial, amt: i32) -> bool {
		self.ore_amount >= amt
	}

	#[export]
	fn consume_ore(&mut self, _base: &Spatial, amt: i32) {
		self.ore_amount = (self.ore_amount - amt).max(0)
	}

	#[export]
	fn get_structure_info(&self, _base: &Spatial, instance_id: i64, minimal: bool) -> String {
		if let Some(stc) = self.structures_by_id.get(&instance_id) {
			let mut info = format!("{}", stc.ty_name());
			if stc.can_be_powered() {
				if stc.is_powered() {
					info += " (powered)";
				}
			}

			if !minimal {
				info += stc.ty_description();
			}

			info
		} else {
			"?".to_string()
		}
	}
}

fn random_positions(n: usize) -> Vec<Vector2> {
	let dist = rand::distributions::Uniform::new(-40.0, 40.0);

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

enum WaterResult {
	NothingToDo,
	WaterConsumed,
	/// If at least one water field depletes, need to recompute pipes
	WaterDepleted,
}
