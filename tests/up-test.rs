use rusqlite::{Connection, params};
use std::{
	fs,
	path::Path,
	process::{Command, Stdio}
};

///REQUIRES miniserve
#[test]
fn up_test() {
	//First ensure the database file doesn't exist
	let db_path = Path::new("./corkdb");
	if db_path.exists() {
		fs::remove_file(db_path).unwrap();
	}

	//Start RSS feed at a first date
	let mut first_feed = Command::new("miniserve")
		.args(&["./assets/sample2.rss"])
		.stdout(Stdio::null())
		.spawn()
		.expect("Miniserve failed");

	//Add the feed to our program
	Command::new("cargo")
		.args(&["run", "--quiet", "--", "add","http://localhost:8080" ])
		.output()
		.expect("Cargo run failed");

	//kill the first feed and start the newer feed
	first_feed.kill();
	let mut second_feed = Command::new("miniserve")
		.args(&["./assets/sample2-next-week.rss"])
		.stdout(Stdio::null())
		.spawn()
		.expect("Miniserve failed");

	//Search for updates
	let output = Command::new("cargo")
		.args(&["run", "--quiet", "--", "up"])
		.output()
		.expect("Cargo run failed");
	let output = std::str::from_utf8(&output.stdout).expect("Output could not be read as a string");
	//The output must always have the link to the item (if it exists)
	assert!(output.contains("http://unique"));

	//Ensure that the database has the new item
	let db = Connection::open("corkdb").unwrap();
	let url_in_database:String = db.prepare("SELECT url FROM items WHERE title_or_desc=(?);") .expect("DB fail")
		.query_map(&["Discussion about recent events"], |row| row.get(0) ) .expect("DB fail")
		.next() .expect("Did not find item in database")
		.expect("DB failed to get row");
	assert_eq!(&url_in_database, "http://unique");

	second_feed.kill();
}
