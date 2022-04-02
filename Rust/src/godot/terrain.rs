use gdnative::{
	api::{ImageTexture, MeshInstance, ShaderMaterial},
	prelude::*,
};
use noise::{NoiseFn, Perlin};

#[derive(NativeClass, Debug)]
#[inherit(Node)]
pub struct Terrain {
	pub mesh: Option<Ref<MeshInstance>>,
	pub image_data: ByteArray,
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
			image_data: ByteArray::from_iter(std::iter::repeat(128u8).take(256 * 256)),
		}
	}

	fn reload_image(&mut self) {
		let image = Image::new().into_shared();
		image.create_from_data(256, 256, false, Image::FORMAT_L8, self.image_data.clone());

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
		self.reload_image()
	}

	#[export]
	fn _process(&mut self, _base: &Node, _dt: f32) {
		let noise = Perlin::new();
		self.image_data = ByteArray::from_iter((0..256).flat_map(|i| {
			(0..256).map(move |j| {
				use std::f64::consts::SQRT_2;
				let noise = noise.get([i as f64 / 16.0, j as f64 / 16.0]);
				(256.0 * (noise + (SQRT_2 / 2.0)) / SQRT_2) as u8
			})
		}));
		self.reload_image();
	}
}
