pub use gdnative::prelude::*;

pub struct World {}

impl World {
    pub fn new() -> World {
        Self {}
    }

    pub fn logic(&mut self, _dt: f32) {
		// do stuff
	}
}
