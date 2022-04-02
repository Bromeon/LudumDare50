use gdnative::prelude::*;
use rand::Rng;

#[derive(NativeClass, Debug, Default)]
#[inherit(Spatial)]
pub struct Spatials {
	#[property]
	pub unimplemented: i32,
}

#[methods]
impl Spatials {
	fn new(_base: &Spatial) -> Self {
		godot_print!("Spatials is instantiated.");

		Self::default()
	}

	#[export]
	fn load(&self, base: &Spatial, scene: Ref<PackedScene>) {
		for _pos in random_positions(16) {
			let instanced = scene.instance(0).unwrap();
			let instanced = instanced.cast::<Spatial>();

			base.add_child(instanced, false);
		}
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
