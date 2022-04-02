use gdnative::{
	api::{ImageTexture, MeshInstance, ShaderMaterial},
	prelude::*,
};
use ndarray::{s, Array2};

use terrain_array::*;

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