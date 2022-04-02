use gdnative::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::objects::Structure;
use crate::VariantType::Int32Array;
use crate::Vector2Ext;

#[derive(NativeClass, Debug, Default)]
#[inherit(Spatial)]
pub struct SpatialObjects {
	structures_by_id: HashMap<i64, Structure>,
}

#[methods]
impl SpatialObjects {
	fn new(_base: &Spatial) -> Self {
		godot_print!("Spatials is instantiated.");

		Self::default()
	}

	#[export]
	fn load(&mut self, base: &Spatial, scene: Ref<PackedScene>) {
		for pos in random_positions(16) {
			let instanced = scene.instance(0).unwrap();
			let instanced = instanced.cast::<Spatial>();
			let id = instanced.get_instance_id();

			instanced.set_translation(pos.to_3d());
			base.add_child(instanced, false);

			let stc = Structure::new(pos);

			godot_print!("Created structure {}: {:?}", id, stc);
			self.structures_by_id.insert(id, stc);
		}
	}

	#[export]
	fn query_radius(&self, _base: &Spatial, position3d: Vector3, radius: f32) -> Vec<i64> {
		self.structures_by_id.keys().copied().collect()
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
