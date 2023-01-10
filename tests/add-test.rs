use rusqlite::{Connection, params};
use std::{
	fs,
	path::Path,
	process::{Command, Stdio}
};

///REQUIRES miniserve
#[test]
fn add_test() {
	//First ensure the database file doesn't exist
	let db_path = Path::new("./corkdb");
	if db_path.exists() {
		fs::remove_file(db_path).unwrap();
	}

	//Start a server to simulate an rss feed
	let mut miniserve = Command::new("miniserve")
		.args(&["./assets/sample1.rss"])
		.stdout(Stdio::null())
		.spawn()
		.expect("Failed to launch miniserve");

	//run our program
	Command::new("cargo")
		.args(&["run", "--quiet", "--", "add", "http://localhost:8080"])
		.output()
		.expect("Cargo run failed");

	let db = Connection::open("corkdb").unwrap();
	let channel_count = db.prepare("SELECT * FROM channels;")
		.unwrap()
		.query_map([], |row| {Ok(1)})
		.unwrap()
		.flatten()
		.fold(0, |ac, n| {ac + n});

	let item_count = db.prepare("SELECT * FROM items;")
		.unwrap()
		.query_map([], |row| {Ok(1)})
		.unwrap()
		.flatten()
		.fold(0, |ac, n| {ac + n});

	assert_eq!(channel_count, 1);
	assert_eq!(item_count, 2);

	miniserve.kill();
}
