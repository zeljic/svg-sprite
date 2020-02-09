#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
};
use xmltree::{Element, EmitterConfig, Namespace, XMLNode};

#[derive(Debug)]
struct SVGFile {
	tree_names: Vec<String>,
	system_path: PathBuf,
}

impl<P> From<P> for SVGFile
where
	P: AsRef<Path>,
{
	fn from(path: P) -> Self {
		SVGFile {
			tree_names: vec![],
			system_path: PathBuf::from(path.as_ref()),
		}
	}
}

fn walk(path: &Path, svgs: &mut Vec<SVGFile>, parents: Vec<String>) -> Result<(), &'static str> {
	if !path.is_dir() {
		return Ok(());
	}

	let mut dirs = vec![];
	let valid_ext = OsStr::new("svg");

	path.read_dir()
		.map_err(|_| "read_dir error")?
		.filter_map(|item| item.ok())
		.map(|item| item.path())
		.for_each(|item| {
			if item.is_dir() {
				dirs.push(item);
			} else {
				if let Some(ext) = item.extension() {
					if ext == valid_ext {
						let mut svg_file = SVGFile::from(item.as_path());

						let mut path = parents.to_owned();

						if let Some(name) = item.file_stem() {
							path.push(name.to_string_lossy().to_string());
						}

						svg_file.tree_names = path;

						svgs.push(svg_file);
					}
				}
			}
		});

	for dir in dirs {
		let mut dir_names = parents.to_owned();

		if let Some(dir_name) = dir.file_name() {
			if let Some(dir_name) = dir_name.to_str() {
				dir_names.push(String::from(dir_name));
			}
		}

		walk(dir.as_path(), svgs, dir_names)?;
	}

	Ok(())
}

fn main() -> Result<(), &'static str> {
	let arg_input = Arg::with_name("INPUT")
		.index(1)
		.required(true)
		.help("Source directory where svg files are located");

	let arg_output = Arg::with_name("OUTPUT")
		.index(2)
		.required(true)
		.help("Output file");

	let arg_separator = Arg::with_name("separator")
		.short("s")
		.long("separator")
		.takes_value(true)
		.default_value("-")
		.help("String placed between each directory in generated id for every SVG file");

	let args_matches = App::new(crate_name!())
		.version(crate_version!())
		.author(crate_authors!())
		.about(crate_description!())
		.args(&[arg_input, arg_output, arg_separator])
		.get_matches();

	println!("{:?}", args_matches);

	let input: String = value_t_or_exit!(args_matches, "INPUT", String);
	let output: String = value_t_or_exit!(args_matches, "OUTPUT", String);
	let separator = args_matches.value_of("separator").unwrap_or("-");

	let source_path: &Path = Path::new(input.as_str());

	if !source_path.is_dir() {
		return Err("Source path is not directory");
	}

	let mut svgs = vec![];

	walk(source_path, &mut svgs, vec![])?;

	let mut svg = Element::new("svg");
	let mut namespaces = Namespace::empty();

	namespaces.put("", "http://www.w3.org/2000/svg");
	namespaces.put("xlink", "http://www.w3.org/1999/xlink");

	svg.namespaces = Some(namespaces);

	svgs.iter().for_each(|svg_file| {
		if let Ok(svg_content) = std::fs::read_to_string(&svg_file.system_path) {
			if let Ok(svg_root_element) = Element::parse(svg_content.as_bytes()) {
				let mut symbol = Element::new("symbol");

				if let Some((k, v)) = svg_root_element.attributes.get_key_value("viewBox") {
					symbol.attributes.insert(k.to_owned(), v.to_owned());
				}

				for child in svg_root_element.children {
					match child {
						XMLNode::Element(_) | XMLNode::CData(_) | XMLNode::Text(_) => {
							symbol.children.push(child)
						}
						_ => {}
					}
				}

				symbol.attributes.insert(
					"id".to_owned(),
					format!("#{}", svg_file.tree_names.join(separator)),
				);

				svg.children.push(XMLNode::Element(symbol));
			}
		}
	});

	if let Ok(file) = std::fs::File::create(output) {
		let mut config = EmitterConfig::default();
		config.perform_indent = true;

		match svg.write_with_config(file, config) {
			Ok(_) => {}
			Err(e) => {
				eprintln!("{:?}", e);
			}
		}
	}

	Ok(())
}
