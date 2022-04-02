use gdnative::prelude::*;
use rand::Rng;
use rstar::{RTree, AABB};
//use std::collections::HashMap;

use crate::objects::Structure;
use crate::{Vector2Ext, Vector3Ext};

#[derive(NativeClass, Debug)]
#[inherit(Spatial)]
pub struct SpatialObjects {
	//structures_by_id: HashMap<i64, Structure>,
	rtree: RTree<Structure>,
}

#[methods]
impl SpatialObjects {
	fn new(_base: &Spatial) -> Self {
		godot_print!("Spatials is instantiated.");

		Self {
			//structures_by_id: HashMap::new(),
			rtree: RTree::new(),
		}
	}

	#[export]
	fn load(&mut self, base: &Spatial, scene: Ref<PackedScene>) {
		let mut structures = vec![];
		for pos in random_positions(1000) {
			let instanced = scene.instance(0).unwrap();
			let instanced = instanced.cast::<Spatial>();
			let id = instanced.get_instance_id();

			instanced.set_translation(pos.to_3d());
			instanced.set_scale(0.2 * Vector3::ONE);
			base.add_child(instanced, false);

			structures.push(Structure::new(pos, id));
		}

		godot_print!("Bulk-add {} structures", structures.len());
		self.rtree = RTree::bulk_load(structures);
	}

	#[export]
	fn query_radius(&self, _base: &Spatial, position3d: Vector3, radius: f32) -> Vec<i64> {
		//self.structures_by_id.keys().copied().collect()
		let half_size = Vector2::ONE * radius;
		let center = position3d.to_2d();
		let p1 = (center - half_size).to_rstar();
		let p2 = (center + half_size).to_rstar();

		let aabb = AABB::from_corners(p1, p2);
		println!("Query {:?}", aabb);

		let radius_sq = radius * radius;
		self.rtree
			.locate_in_envelope(&aabb)
			.filter(|stc| stc.position().distance_squared_to(center) < radius_sq)
			.map(|stc| stc.instance_id())
			.collect()
	}

	#[allow(dead_code)]
	fn add_structure(&mut self, stc: Structure) {
		godot_print!("Add structure {:?}", stc);

		//self.structures_by_id.insert(id, stc);
		self.rtree.insert(stc);
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
