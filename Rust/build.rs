extern crate core;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use heck::{ToSnakeCase, ToUpperCamelCase};

macro_rules! class {
	($name:ident) => {
		Class {
			name: stringify!($name).to_string(),
			base: "Reference".to_string(),
		}
	};
	($name:ident : $base:ident) => {
		Class {
			name: stringify!($name).to_string(),
			base: stringify!($base).to_string(),
		}
	};
}

fn main() {
	let cfg = NativeClasses {
		godot_gdns_dir: PathBuf::from("../Godot/Native"),
		godot_gdnlib_res_path: PathBuf::from("res://Native/NativeLib.gdnlib"),
		rust_class_dir: PathBuf::from("src/godot"),
		classes: vec![
			class!(GodotApi: Node),
			class!(LittleStruct),
			class!(Terrain : Node),
			class!(Spatials : Spatial)
		],
	};

	sync(cfg).expect("Sync configured correctly");
}

// ----------------------------------------------------------------------------------------------------------------------------------------------

fn sync(cfg: NativeClasses) -> Result<(), std::io::Error> {
	validate(&cfg)?;

	fs::create_dir_all(&cfg.godot_gdns_dir)?;
	fs::create_dir_all(&cfg.rust_class_dir)?;

	let mut camel_to_snake = HashMap::new();
	for class in cfg.classes.iter() {
		camel_to_snake.insert(class.name.as_str(), class.name.to_snake_case());
	}

	// Remove no longer needed .gdns native scripts
	for gdns_file in fs::read_dir(&cfg.godot_gdns_dir)? {
		let path = gdns_file?.path();
		let filename = path.file_name().unwrap().to_str().unwrap();

		if let Some(camel_name) = filename.strip_suffix(".gdns") {
			if !camel_to_snake.contains_key(&camel_name) {
				//panic!("remove {path:?}");
				fs::remove_file(path)?;
			}
		}
	}

	// Remove no longer needed Rust files
	for rust_file in fs::read_dir(&cfg.rust_class_dir)? {
		let path = rust_file?.path();
		let filename = path.file_name().unwrap().to_str().unwrap();

		if let Some(snake_name) = filename.strip_suffix(".rs") {
			let camel_name = snake_name.to_upper_camel_case();
			if snake_name != "mod" && !camel_to_snake.contains_key(camel_name.as_str()) {
				//dbg!(camel_name);
				//panic!("remove {path:?}, map={camel_to_snake:?}");
				fs::remove_file(path)?;
			}
		}
	}

	// Create new Rust and .gdns files
	for class in cfg.classes.iter() {
		let gdns_path = cfg.godot_gdns_dir.join(class.name.clone() + ".gdns");
		if !gdns_path.exists() {
			fs::write(gdns_path, make_gdns(&cfg, &class))?;
		}

		let snake_class_name = class.name.to_snake_case();
		let rust_path = cfg.rust_class_dir.join(snake_class_name + ".rs");

		if !rust_path.exists() {
			//panic!("not exists: {rust_path:?}");
			fs::write(&rust_path, make_rust_class(&class))?;
		}
	}

	let mod_path = cfg.rust_class_dir.join("mod.rs");

	// Below statement causes recompilation every time, since build.rs overwrites structs/mod.rs repeatedly
	//println!("cargo:rerun-if-changed={}", mod_path.display());

	fs::write(mod_path, make_rust_mod(&cfg.classes))?;

	Ok(())
}

fn make_gdns(cfg: &NativeClasses, class: &Class) -> String {
	format!(
		r#"[gd_resource type="NativeScript" load_steps=2 format=2]

[ext_resource path="{gdnlib}" type="GDNativeLibrary" id=1]

[resource]
resource_name = "{name}"
class_name = "{name}"
library = ExtResource( 1 )
script_class_name = "{name}"
"#,
		gdnlib = cfg.godot_gdnlib_res_path.display(),
		name = class.name
	)
}

fn make_rust_class(class: &Class) -> String {
	// api
	let base = format!("{base}", base = class.base);
	let inherit = if class.base != "Reference" {
		format!("\n#[inherit({base})]", base = base)
	} else {
		String::new()
	};

	format!(
		r#"use gdnative::prelude::*;

#[derive(NativeClass, Debug, Default)]{inherit}
pub struct {class} {{
	#[property]
	pub unimplemented: i32,
}}

#[methods]
impl {class} {{
	fn new(_base: &{base}) -> Self {{
		Self::default()
	}}
}}
"#,
		class = class.name,
		base = base,
		inherit = inherit,
	)
}

fn make_rust_mod(classes: &[Class]) -> String {
	let mut mods = String::new();
	let mut uses = String::new();
	let mut registers = String::new();

	for class in classes.iter() {
		let snake_name = class.name.to_snake_case();
		mods += &format!("\nmod {};", snake_name);
		uses += &format!("\npub use {}::*;", snake_name);
		registers += &format!("\n\thandle.add_class::<{}>();", class.name);
	}

	format!(
		r#"// Auto-generated; do not edit.
{mods}
{uses}

pub fn register_classes(handle: gdnative::init::InitHandle) {{{registers}
}}
"#,
		mods = mods,
		uses = uses,
		registers = registers,
	)
}

fn validate(cfg: &NativeClasses) -> Result<(), std::io::Error> {
	if !cfg.godot_gdnlib_res_path.starts_with("res://") {
		error(".gdnlib path must be a Godot 'res://' path")
	} else {
		Ok(())
	}
}

fn error(message: &str) -> Result<(), std::io::Error> {
	Err(std::io::Error::new(std::io::ErrorKind::Other, message))
}

struct NativeClasses {
	pub godot_gdns_dir: PathBuf,
	pub godot_gdnlib_res_path: PathBuf,
	pub rust_class_dir: PathBuf,
	pub classes: Vec<Class>,
}

struct Class {
	pub name: String,
	pub base: String,
}
