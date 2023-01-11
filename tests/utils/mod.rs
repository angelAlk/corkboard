use std::{
	fs,
	path::Path,
	process::{Child, Command, Stdio, Output}
};

///Deletes the database file if it's present
pub fn ensure_new_database() {
	let db_path = Path::new("./corkdb");
	if db_path.exists() {
		fs::remove_file(db_path).unwrap();
	}
}

///Launches the program miniserve to act as an http server
///for our rss feeds in assets
pub fn launch_miniserve(feed_file: &str, extra_args:Option<&[&str]>) -> Child {
	Command::new("miniserve")
		.args(&[feed_file])
		.args(extra_args.unwrap_or(&[]))
		.stdout(Stdio::null())//Don't display miniserve output
		.spawn()
		.expect("Failed to launch miniserve")
}

///Run the application
pub fn run_cork(parameters: &[&str]) -> Output {
	Command::new("cargo")
		.args(&["run", "--quiet", "--"])
		.args(parameters)
		.output()
		.expect("Cargo run failed")
}
