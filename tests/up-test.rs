use rusqlite::Connection;
mod utils;
use utils::*;

#[test]
fn up_test() {
	ensure_new_database();

	//start the old feed
	let mut first_feed = launch_miniserve("./assets/sample2.rss", None);

	//Add the feed to our program
	run_cork(&["add","http://localhost:8080"]);

	//Release the port for the newer feed
	first_feed.kill().unwrap();
	let mut newer_feed = launch_miniserve("./assets/sample2-next-week.rss", None);

	//Search for updates
	let output = run_cork(&["up"]);
	let output = std::str::from_utf8(&output.stdout).expect("Could not read output as string");
	//The output must always have the link to the item (if it exists)
	assert!(output.contains("http://unique"));

	//Ensure that the database has the new item
	let db = Connection::open("corkdb").unwrap();
	let url_in_database:String = db.prepare("SELECT url FROM items WHERE title_or_desc=(?);") .expect("DB fail")
		.query_map(&["Discussion about recent events"], |row| row.get(0) ) .expect("DB fail")
		.next() .expect("Did not find item in database")
		.expect("DB failed to get row");
	assert_eq!(&url_in_database, "http://unique");

	newer_feed.kill().unwrap();
}
