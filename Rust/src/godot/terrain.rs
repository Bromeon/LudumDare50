use gdnative::{
	api::{ImageTexture, MeshInstance, ShaderMaterial},
	prelude::*,
};
use ndarray::{s, Array2};
use noise::{NoiseFn, Perlin};

#[derive(NativeClass, Debug)]
#[inherit(Node)]
pub struct Terrain {
	pub mesh: Option<Ref<MeshInstance>>,
	array: TerrainArray,
}

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

#[methods]
impl Terrain {
	fn new(_base: &Node) -> Self {
		Self {
			mesh: None,
			array: TerrainArray::default(),
		}
	}

	fn reload_image(&mut self) {
		let image = Image::new().into_shared();
		let bytes = ByteArray::from_slice(self.array.data().slice(s![.., ..]).as_slice().unwrap());
		image.create_from_data(256, 256, false, Image::FORMAT_L8, bytes);

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

	#[export]
	fn _ready(&mut self, base: &Node) {
		let mesh = get_node!(base, "Mesh", MeshInstance);
		self.mesh = Some(mesh);

		self.array.fill_shape(
			Shape::Circle {
				center: [50, 50],
				radius: 10,
			},
			BLIGHT,
		);

		self.reload_image()
	}

	#[export]
	fn _process(&mut self, _base: &Node, _dt: f32) {
		self.array.dilate();
		self.reload_image();
	}
}

#[derive(Debug)]
pub struct TerrainArray {
	data: ndarray::Array2<u8>,
}

pub const BLIGHT: u8 = u8::MAX;
pub const CLEAN: u8 = 0u8;

pub enum Shape {
	Circle { center: [usize; 2], radius: usize },
}

impl TerrainArray {
	pub const WIDTH: usize = 256;
	pub const HEIGHT: usize = 256;

	pub fn new() -> Self {
		Self {
			data: ndarray::Array2::from_elem((Self::WIDTH, Self::HEIGHT), CLEAN),
		}
	}

	pub fn fill_shape(&mut self, shape: Shape, fill: u8) {
		match shape {
			Shape::Circle { center, radius } => {
				let window_size = radius * 2 + 1;
				let wi = center[0].saturating_sub(radius);
				let wj = center[1].saturating_sub(radius);
				self.data
					.slice_mut(s![wi..wi + window_size, wj..wj + window_size])
					.indexed_iter_mut()
					.for_each(|((i, j), value)| {
						let di = radius as isize - i as isize;
						let dj = radius as isize - j as isize;
						let dist_sq = (di * di + dj * dj) as usize;
						if dist_sq <= radius * radius {
							*value = fill
						}
					});
			}
		}
	}

	pub fn dilate(&mut self) {
		let mut new_data = ndarray::Array2::zeros(self.data.raw_dim());
		ndarray::Zip::from(new_data.slice_mut(s![2..254, 2..254]))
			.and(self.data.windows((5, 5)))
			.for_each(|v, window| {
				let kernel = ndarray::array![
					[0, 1, 1, 1, 0],
					[1, 1, 1, 1, 1],
					[1, 1, 1, 1, 1],
					[1, 1, 1, 1, 1],
					[0, 1, 1, 1, 0],
				];
				*v = ndarray::Zip::from(window)
					.and(&kernel)
					.fold(
						CLEAN,
						|acc, val, k| acc.max(*val * k),
					);
			});
		self.data = new_data;
	}

	pub fn data(&self) -> &Array2<u8> {
		&self.data
	}
}

impl Default for TerrainArray {
	fn default() -> Self {
		Self::new()
	}
}
