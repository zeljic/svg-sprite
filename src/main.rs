#[macro_use]
extern crate clap;

use anyhow::Context;
use clap::{App, Arg};
use std::{
	ffi::OsStr,
	io::Write,
	path::{Path, PathBuf},
};
use xmltree::{Element, EmitterConfig, Namespace, XMLNode};

#[derive(Debug, PartialEq)]
enum SVGTag {
	G,
	SYMBOL,
}

#[derive(Debug)]
struct SVGFile {
	tree_names: Vec<String>,
	system_path: PathBuf,
}

impl SVGFile {
	fn print(&self, writer: &mut dyn Write) {
		write!(writer, "nice").unwrap();
	}
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

fn walk(path: &Path, svgs: &mut Vec<SVGFile>, tree_names: Vec<String>, recursive: bool) -> anyhow::Result<()> {
	if !path.is_dir() {
		return Err(anyhow::anyhow!("path must be directory")).with_context(|| format!("{:?}", path));
	}

	let mut dirs = vec![];
	let valid_ext = OsStr::new("svg");

	path.read_dir()
		.with_context(|| format!("{:?}", path))?
		.filter_map(|item| item.ok())
		.map(|item| item.path())
		.for_each(|item| {
			if recursive && item.is_dir() {
				dirs.push(item);
			} else if let Some(ext) = item.extension() {
				if ext == valid_ext {
					let mut svg_file = SVGFile::from(item.as_path());
					let mut path = tree_names.to_owned();

					if let Some(name) = item.file_stem() {
						path.push(name.to_string_lossy().to_string());
					}

					svg_file.tree_names = path;

					svgs.push(svg_file);
				}
			}
		});

	if recursive {
		for dir in dirs {
			let mut dir_names = tree_names.to_owned();

			if let Some(dir_name) = dir.file_name() {
				if let Some(dir_name) = dir_name.to_str() {
					dir_names.push(dir_name.to_owned());
				}
			}

			walk(dir.as_path(), svgs, dir_names, recursive)?;
		}
	}

	Ok(())
}

fn main() {
	let arg_input = Arg::with_name("INPUT")
		.index(1)
		.required(true)
		.long_help("Source directory where svg files are located");

	let arg_output = Arg::with_name("OUTPUT").index(2).required(false).long_help("Output file");

	let arg_separator = Arg::with_name("separator")
		.short("s")
		.long("separator")
		.takes_value(true)
		.default_value("-")
		.long_help("String placed between each directory in generated id for every SVG file");

	let arg_tag = Arg::with_name("tag")
		.short("t")
		.long("tag")
		.takes_value(true)
		.default_value("symbol")
		.possible_values(&["g", "symbol"])
		.long_help("Tag for every generated child of new created SVG file");

	let arg_remove_attributes = Arg::with_name("remove-attribute")
		.short("a")
		.long("remove-attribute")
		.takes_value(true)
		.multiple(true)
		.number_of_values(1)
		.long_help("Remove attributes from SVG file. It works only on first level elements.");

	let arg_remove_elements = Arg::with_name("remove-element")
		.short("e")
		.long("remove-element")
		.takes_value(true)
		.multiple(true)
		.number_of_values(1)
		.long_help("Remove elements from svg based on tag name. It works only on first level.");

	let arg_recursive = Arg::with_name("recursive")
		.short("r")
		.long("recursive")
		.long_help("Get files from INPUT recursively");

	let arg_verbose = Arg::with_name("verbose")
		.short("v")
		.multiple(true)
		.long_help("Show more info about files and generated SVG file");

	let args_matches = App::new(crate_name!())
		.version(crate_version!())
		.author(crate_authors!())
		.about(crate_description!())
		.args(&[
			arg_input,
			arg_output,
			arg_separator,
			arg_tag,
			arg_remove_attributes,
			arg_remove_elements,
			arg_recursive,
			arg_verbose,
		])
		.get_matches();

	let input = value_t!(args_matches, "INPUT", String).unwrap_or_else(|e| e.exit());
	let output = args_matches.value_of("OUTPUT");
	let separator = value_t!(args_matches, "separator", String).unwrap_or_else(|_| "-".to_owned());
	let tag: SVGTag = match args_matches.value_of("tag") {
		Some("g") => SVGTag::G,
		_ => SVGTag::SYMBOL,
	};
	let remove_attributes: Vec<String> = values_t!(args_matches, "remove-attribute", String).unwrap_or_else(|_| vec![]);
	let remove_elements: Vec<String> = values_t!(args_matches, "remove-element", String).unwrap_or_else(|_| vec![]);
	let recursive = args_matches.is_present("recursive");

	let input_path: &Path = Path::new(input.as_str());

	let mut svgs = vec![];

	if let Err(e) = walk(input_path, &mut svgs, vec![], recursive) {
		println!("{:#?}", e);

		return;
	};

	let mut svg = Element::new("svg");
	let mut namespaces = Namespace::empty();

	namespaces.put("", "http://www.w3.org/2000/svg");
	namespaces.put("xlink", "http://www.w3.org/1999/xlink");

	svg.namespaces = Some(namespaces);

	svgs.iter().for_each(|svg_file| {
		if let Ok(svg_content) = std::fs::read_to_string(&svg_file.system_path) {
			if let Ok(svg_root_element) = Element::parse(svg_content.as_bytes()) {
				let mut new_svg_element = Element::new(match tag {
					SVGTag::G => "g",
					SVGTag::SYMBOL => "symbol",
				});

				if tag == SVGTag::SYMBOL {
					if let Some((k, v)) = svg_root_element.attributes.get_key_value("viewBox") {
						new_svg_element.attributes.insert(k.to_owned(), v.to_owned());
					}
				}

				svg_root_element
					.children
					.into_iter()
					.filter(|child| match child {
						XMLNode::Comment(_) => false,
						_ => true,
					})
					.filter(|child| {
						if let XMLNode::Element(el) = child {
							!remove_elements.contains(&el.name)
						} else {
							true
						}
					})
					.for_each(|mut child| {
						if let XMLNode::Element(el) = &mut child {
							let attributes = &mut el.attributes;

							for attribute in &remove_attributes {
								attributes.remove(attribute);
							}
						}

						new_svg_element.children.push(child);
					});

				new_svg_element
					.attributes
					.insert("id".to_owned(), svg_file.tree_names.join(&separator));

				svg.children.push(XMLNode::Element(new_svg_element));
			}
		}
	});

	let mut output_location: Option<Box<dyn Write>> = None;

	if let Some(output) = output {
		match std::fs::File::create(output) {
			Ok(output) => {
				output_location = Some(Box::new(output));
			}
			Err(e) => {
				println!("Unable to write to output file. {}", e);
			}
		}
	} else {
		output_location = Some(Box::new(std::io::stdout()));
	}

	if let Some(output_location) = output_location {
		let mut config = EmitterConfig::default();
		config.perform_indent = true;

		if let Err(e) = svg.write_with_config(output_location, config) {
			println!("Unable to write SVG. {}", e);
		}
	}
}
