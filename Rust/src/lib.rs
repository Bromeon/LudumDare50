use gdnative::prelude::*;

//#[path="godot/structs/mod.rs"]
pub mod godot;
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

#[allow(dead_code)]
fn compile_test() {
	let _ = godot::LittleStruct { unimplemented: 77 };

}