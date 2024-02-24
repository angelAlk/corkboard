use std::{
	env,
	fs,
	path::Path,
	process::{Child, Command, Stdio, Output}
};

use rusqlite::Connection;
use sha2::{Sha256, Digest};

///Deletes the database file if it's present
pub fn ensure_new_database() {
	let db_path = Path::new("./corkdb");
	if db_path.exists() {
		fs::remove_file(db_path).unwrap();
	}
}

///An instance of the miniserve program we are using to
///deliver the test RSS feeds.
pub struct Miniserve (Child);
impl Miniserve {
	///Start a miniserve instance
	pub fn launch(feed_file: &str, extra_args:Option<&[&str]>) -> Self {
		Miniserve(
			Command::new("miniserve")
				.args(&[feed_file])
				.args(extra_args.unwrap_or(&[]))
				.stdout(Stdio::null())//Don't display miniserve output
				.spawn()
				.expect("Failed to launch miniserve")
		)
	}

	///Kill the miniserve process.
	///this operation frees the port passed in the launch call.
	pub fn kill(&mut self) {
		self.0.kill().unwrap()
	}
}
impl Drop for Miniserve {
	fn drop(&mut self)  {
		self.kill();
	}
}

///Run the application
pub fn run_cork(parameters: &[&str]) -> Output {
	Command::new("cargo")
		.args(&["run", "--quiet", "--"])
		.args(parameters)
		//Setting $CORKDB_TEST to True, so that corkboard will use "./corkdb" as the database path
		.env("CORKDB_TEST", "true")
		.output()
		.expect("Cargo run failed")
}

///Count the channels in the database
pub fn count_channels(db: &Connection) -> i64 {
	let db_result:Vec<_> = db.prepare("SELECT COUNT(*) FROM channels;").unwrap()
		.query_map([], |row| row.get(0)).unwrap()
		.flatten()
		.collect();
	db_result[0]
}

///Count the items in the database
pub fn count_items(db: &Connection) -> i64 {
	let db_result:Vec<_> = db.prepare("SELECT COUNT(*) FROM items;").unwrap()
		.query_map([], |row| row.get(0)).unwrap()
		.flatten()
		.collect();
	db_result[0]
}

///Hashes a string and formats it in the same way as the constructor in the rss module.
pub fn hash_string(s:&str) -> String {
	let mut h = Sha256::new();
	h.update(s.as_bytes());
	let hash = h.finalize();
	format!("{:016x}", hash)
}
