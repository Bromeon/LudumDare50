use gdnative::prelude::*;

// Macros need to be defined before the modules that use them.
#[macro_export]
macro_rules! get_node {
	($base:ident, $path:expr, $typ:ty) => {
		unsafe {
			$base
				.get_node($path)
				.unwrap()
				.cast::<$typ>()
				.assume_shared()
		}
	};
}

//#[path="godot/structs/mod.rs"]
pub mod godot;
pub mod objects;

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
trait Vector2Ext {
	fn to_rstar(self) -> [f32; 2];
	fn to_3d(self) -> Vector3;
}

impl Vector2Ext for Vector2 {
	fn to_rstar(self) -> [f32; 2] {
		[self.x, self.y]
	}

	fn to_3d(self) -> Vector3 {
		Vector3::new(self.x, 0.0, self.y)
	}
}

trait Vector3Ext {
	fn to_rstar(self) -> [f32; 2];
	fn to_2d(self) -> Vector2;
}

impl Vector3Ext for Vector3 {
	// Note: 2D point is intended
	fn to_rstar(self) -> [f32; 2] {
		[self.x, self.z]
	}

	fn to_2d(self) -> Vector2 {
		Vector2::new(self.x, self.z)
	}
}
