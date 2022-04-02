use gdnative::prelude::*;

//#[path="godot/structs/mod.rs"]
pub mod godot;
pub mod objects;
pub mod world;

use godot::register_classes;

godot_init!(register_classes);

trait MyDisplay {
	fn str(&self) -> String;
}

impl MyDisplay for Vector2 {
	fn str(&self) -> String {
		format!("({}, {})", self.x, self.y)
	}
}

// Add extension functions here
trait VectorExt {
	fn to_rstar(self) -> [f32; 2];
	fn to_3d(self) -> Vector3;
}

impl VectorExt for Vector2 {
	fn to_rstar(self) -> [f32; 2] {
		[self.x, self.y]
	}

	fn to_3d(self) -> Vector3 {
		Vector3::new(self.x, 0.0, self.y)
	}
}

#[allow(dead_code)]
fn compile_test() {
	let _ = godot::LittleStruct { unimplemented: 77 };
}
