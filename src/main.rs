use std::path::Path;

fn main() -> Result<(), &'static str> {
	let args = std::env::args().skip(1).collect::<Vec<String>>();
	let source = args.get(0).ok_or_else(|| "No source path")?;
	let destination = args.get(1).ok_or_else(|| "No destination path");

	let source_path: &Path = Path::new(source);

	if !source_path.is_dir() {
		return Err("Source path is not directory");
	}

	Ok(())
}
