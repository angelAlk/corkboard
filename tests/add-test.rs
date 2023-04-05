use rusqlite::Connection;
mod utils;
use utils::*;

#[test]
fn add_test() {
	ensure_new_database();

	let mut feed_source = Miniserve::launch("./assets/sample1.rss", None);
	run_cork(&["add", "http://localhost:8080"]);
	feed_source.kill();

	let db = Connection::open("corkdb").unwrap();

	let channel_count = count_channels(&db);
	let item_count = count_items(&db);

	assert_eq!(channel_count, 1);
	assert_eq!(item_count, 2);
}