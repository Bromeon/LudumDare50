use gdnative::{
	api::{ImageTexture, MeshInstance, PlaneMesh, ShaderMaterial},
	prelude::*,
};
use ndarray::s;

use terrain_array::*;

#[derive(NativeClass, Debug)]
#[inherit(Node)]
pub struct Terrain {
	pub mesh: Option<Ref<MeshInstance>>,
	array: TerrainArray,
	frame_count: usize,
	measurements: PlaneMeasurements,
}

#[derive(Debug, Default)]
struct PlaneMeasurements {
	top_left: Vector2,
	plane_size: Vector2,
}

trait Vec3Ext {
	fn xz(&self) -> Vector2;
}
impl Vec3Ext for Vector3 {
	fn xz(&self) -> Vector2 {
		Vector2::new(self.x, self.z)
	}
}

#[methods]
impl Terrain {
	fn new(_base: &Node) -> Self {
		Self {
			mesh: None,
			array: TerrainArray::default(),
			frame_count: 0,
			measurements: Default::default(), // Will initialize later
		}
	}

	fn reload_image(&mut self) {
		let image = Image::new().into_shared();
		let bytes = ByteArray::from_slice(self.array.data().slice(s![.., ..]).as_slice().unwrap());
		image.create_from_data(
			TerrainArray::WIDTH as i64,
			TerrainArray::HEIGHT as i64,
			false,
			Image::FORMAT_L8,
			bytes,
		);

		let material = self
			.mesh
			.unwrap()
			.get_surface_material(0)
			.unwrap()
			.cast::<ShaderMaterial>();

		let texture = material
			.get_shader_param("Splatmap")
			.try_to_object::<ImageTexture>()
			.unwrap();
		texture.create_from_image(image, Texture::FLAGS_DEFAULT);
	}

	fn compute_measurements(&self) -> PlaneMeasurements {
		let mesh = self.mesh.unwrap();
		let plane_size = mesh.mesh().unwrap().cast::<PlaneMesh>().size();
		let center = mesh.transform().origin.xz();
		let top_left = center - plane_size / 2.0;
		PlaneMeasurements {
			top_left,
			plane_size,
		}
	}

	#[export]
	fn _ready(&mut self, base: &Node) {
		let mesh = get_node!(base, "Mesh", MeshInstance);
		self.mesh = Some(mesh);

		self.array.fill_shape(
			Shape::Circle {
				center: [150, 150],
				radius: 40,
			},
			BLIGHT,
		);

		self.measurements = self.compute_measurements();

		self.reload_image()
	}

	#[export]
	fn _physics_process(&mut self, _base: &Node, _dt: f32) {
		self.frame_count += 1;
		if self.frame_count % 7 == 0 {
			self.array.dilate();
		}
		self.reload_image();
	}

	/// Given a position in world coordinates, returns its position inside the
	/// inner `array`.
	fn world2grid(&self, world_pos: Vector3) -> [usize; 2] {
		let normalized =
			(world_pos.xz() - self.measurements.top_left) / self.measurements.plane_size;
		let grid =
			normalized * Vector2::new(TerrainArray::WIDTH as f32, TerrainArray::HEIGHT as f32);
		[grid.x as usize, grid.y as usize]
	}

	/// Given a position in using the inner array's coordinates, returns the
	/// world position of that point.
	#[allow(dead_code)]
	fn grid2world(&self, pos: [usize; 2]) -> Vector2 {
		let posv2 = Vector2::new(pos[0] as f32, pos[1] as f32);
		let normalized = posv2
			* Vector2::new(
				1.0 / TerrainArray::WIDTH as f32,
				1.0 / TerrainArray::HEIGHT as f32,
			);
		(normalized + self.measurements.top_left) * self.measurements.plane_size
	}

	/// Returns the average blight value (between 0 and 255) of the circle with
	/// given `center` and `radius` values.
	pub fn get_average_blight_in_circle(&self, center: Vector3, radius: f32) -> u8 {
		let center_grid = self.world2grid(center);
		let radius_grid = ((radius / self.measurements.plane_size.x) * 256.0) as usize;
		let circle = Shape::Circle {
			center: center_grid,
			radius: radius_grid,
		};
		self.array.query_shape_avg(circle)
	}

	/// Cleans a circle from blight
	#[export]
	pub fn clean_circle(&mut self, _base: &Node, center: Vector3, radius: f32) {
		let center_grid = self.world2grid(center);
		let radius_grid = ((radius / self.measurements.plane_size.x) * 256.0) as usize;
		let circle = Shape::Circle {
			center: center_grid,
			radius: radius_grid,
		};
		self.array.fill_shape(circle, CLEAN);
	}
}
