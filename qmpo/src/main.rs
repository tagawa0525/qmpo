use clap::Parser;
use qmpo_core::DirectoryUri;

#[derive(Parser, Debug)]
#[command(name = "qmpo")]
#[command(about = "Directory URI handler - opens directories in file manager")]
#[command(version)]
struct Args {
    /// The directory URI to open (e.g., directory:///home/user)
    uri: String,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = run(&args.uri) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(uri_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let uri = DirectoryUri::parse(uri_str)?;
    let path = uri.path();

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }

    // If path is a file, open its parent directory
    let dir = if path.is_file() {
        path.parent()
            .ok_or_else(|| format!("Could not get parent directory of: {}", path.display()))?
    } else {
        path
    };

    showfile::show_path_in_file_manager(dir);

    Ok(())
}
