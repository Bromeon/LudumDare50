use gdnative::{api::Camera, prelude::*};

use crate::{get_node, Vector2Ext};

#[derive(NativeClass, Debug)]
#[inherit(Spatial)]
pub struct Zeppelin {
	/// The camera that will follow this zeppelin. It's stored as a sibling node
	/// called 'ZeppelinCamera'
	pub camera: Option<Ref<Camera>>,
	/// An extra spatial node so we can rotate the zeppelin mesh, consisting of
	/// multiple meshes, independently.
	pub pivot: Option<Ref<Spatial>>,
	pub acceleration: Vector2,
	pub velocity: Vector2,
	pub look_dir: Vector2,
	#[property]
	pub drag: f32,
	#[property]
	pub acc_factor: f32,
	#[property]
	pub cam_acc_factor: f32,
	#[property]
	pub cam_angle: Vector3,
}

#[methods]
impl Zeppelin {
	fn new(_base: &Spatial) -> Self {
		Self {
			camera: None,
			pivot: None,
			acceleration: Vector2::ZERO,
			velocity: Vector2::ZERO,
			look_dir: Vector2::ZERO,
			drag: 0.95,
			acc_factor: 1.0,
			cam_angle: Vector3::new(0.0, -7.0, -4.0),
			cam_acc_factor: 3.0,
		}
	}

	#[export]
	fn _ready(&mut self, base: &Spatial) {
		self.camera = Some(get_node!(base, "../ZeppelinCamera", Camera));
		self.pivot = Some(get_node!(base, "Pivot", Spatial));
	}

	fn process_input(&mut self) {
		let input = Input::godot_singleton();
		let y =
			input.get_action_strength("back", false) - input.get_action_strength("forward", false);
		let x =
			input.get_action_strength("right", false) - input.get_action_strength("left", false);

		self.acceleration = Vector2::new(x as f32, y as f32) * self.acc_factor;
	}

	fn update_camera(&mut self, base: &Spatial, dt: f32) {
		let zeppelin_pos = base.transform().origin;
		let camera = self.camera.unwrap();
		let camera_pos = camera.transform().origin;
		let target = zeppelin_pos - self.cam_angle;

		camera.set_translation(camera_pos.linear_interpolate(target, dt * self.cam_acc_factor));
		camera.look_at(zeppelin_pos, Vector3::UP);
	}

	fn integrate_velocity(&mut self, base: &Spatial, dt: f32) {
		self.velocity += self.acceleration * dt;
		self.velocity *= self.drag;
		base.translate(self.velocity.to_3d() * dt);
	}

	fn rotate_pivot(&mut self, dt: f32) {
		self.look_dir = self.look_dir.linear_interpolate(self.velocity, dt * 1.0);
		let pivot = self.pivot.unwrap();
		if self.look_dir.length_squared() > 0.0 {
			pivot.look_at(
				pivot.global_transform().origin + self.look_dir.to_3d(),
				Vector3::UP,
			)
		}
	}

	#[export]
	fn _physics_process(&mut self, base: &Spatial, dt: f32) {
		self.process_input();
		self.integrate_velocity(base, dt);
		self.update_camera(base, dt);
		self.rotate_pivot(dt);
	}
}
