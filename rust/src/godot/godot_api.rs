use gdnative::prelude::*;
use crate::world::World;

// ----------------------------------------------------------------------------------------------------------------------------------------------

#[derive(NativeClass)]
#[inherit(Node)]
pub struct GodotApi {
	world: Option<World>, // late-init
}


#[methods]
impl GodotApi {
	fn new(_owner: &Node) -> Self {
		Self {
			world: None,
		}
	}

	#[export]
	fn initialize(
		&mut self,
		_owner: &Node,
		string: String
	) {
		self.world = Some(World::new());
		godot_print!("Hello from Rust: {string}")
	}

	#[export]
	fn logic_tick(&mut self, _owner: &Node, dt: f32) {
		self.world().logic(dt)
	}

	fn world(&mut self) -> &mut World {
		self.world.as_mut().expect("World was not initialized yet")
	}
}
