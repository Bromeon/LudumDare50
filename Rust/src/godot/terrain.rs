use gdnative::{
	api::{GlobalConstants, ImageTexture, Mesh, MeshInstance, PlaneMesh, ShaderMaterial},
	prelude::*,
};

#[derive(NativeClass, Debug, Default)]
#[inherit(Node)]
pub struct Terrain {
	#[property]
	pub unimplemented: i32,

	pub mesh: Option<Ref<MeshInstance>>,
}

macro_rules! get_node {
	($base:ident, $path:expr, $typ:ty) => {
		unsafe {
			$base
				.get_node($path)
				.unwrap()
				.cast::<$typ>()
				.unwrap()
				.assume_shared()
		}
	};
}

#[methods]
impl Terrain {
	fn new(_base: &Node) -> Self {
		Self::default()
	}

	#[export]
	fn _ready(&mut self, base: &Node) {
		let mesh = get_node!(base, "Mesh", MeshInstance);
		self.mesh = Some(mesh);

		let data = ByteArray::from_iter(std::iter::repeat(128u8).take(256 * 256));
		let image = Image::new().into_shared();
		image.create_from_data(256, 256, false, Image::FORMAT_L8, data);

		let material = mesh
			.get_surface_material(0)
			.unwrap()
			.cast::<ShaderMaterial>()
			.unwrap();
		dbg!(&material.get_shader_param("Splatmap"));
		let texture = material
			.get_shader_param("Splatmap")
			.try_to_object::<ImageTexture>()
			.unwrap();
		texture.create_from_image(image, Texture::FLAGS_DEFAULT);
	}
}
