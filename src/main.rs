use std::{convert::TryFrom, ffi::OsStr, path::Path};
use xmltree::{Element, Namespace};

#[derive(Debug)]
struct SVGFile {
	path: Vec<String>,
	content: String,
}

impl TryFrom<&Path> for SVGFile {
	type Error = &'static str;

	fn try_from(value: &Path) -> Result<Self, Self::Error> {
		if let Ok(content) = std::fs::read_to_string(value) {
			Ok(SVGFile {
				path: vec![],
				content,
			})
		} else {
			Err("Unable to read file")
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
						if let Ok(mut svg_file) = SVGFile::try_from(item.as_path()) {
							let mut path = parents.to_owned();

							svg_file.path.append(&mut path);

							if let Some(name) = item.file_stem() {
								svg_file.path.push(name.to_string_lossy().to_string());
							}

							svgs.push(svg_file);
						}
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
	let args = std::env::args().skip(1).collect::<Vec<String>>();
	let source = args.get(0).ok_or_else(|| "No source path")?;
	let destination = args.get(1).ok_or_else(|| "No destination path")?;

	let source_path: &Path = Path::new(source);

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

	if let Ok(file) = std::fs::File::create(destination) {
		match svg.write(file) {
			Ok(_) => {}
			Err(e) => {
				eprintln!("{:?}", e);
			}
		}
	}

	Ok(())
}
