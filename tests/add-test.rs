use rusqlite::Connection;
mod utils;
use utils::*;

#[test]
fn add_test() {
	ensure_new_database();
	let mut feed_source = launch_miniserve("./assets/sample1.rss", None);

	run_cork(&["add", "http://localhost:8080"]);

	let db = Connection::open("corkdb").unwrap();
	let channel_count = db.prepare("SELECT * FROM channels;").unwrap()
		.query_map([], |_row| {Ok(1)}).unwrap()
		.flatten()
		.fold(0, |ac, n| {ac + n});
	let item_count = db.prepare("SELECT * FROM items;").unwrap()
		.query_map([], |_row| {Ok(1)}).unwrap()
		.flatten()
		.fold(0, |ac, n| {ac + n});

	assert_eq!(channel_count, 1);
	assert_eq!(item_count, 2);

	feed_source.kill().unwrap();
}
