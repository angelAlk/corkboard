use rusqlite::Connection;
mod utils;
use utils::*;

fn get_channel_count(db: &Connection) -> i64 {
	let db_result:Vec<_> = db.prepare("SELECT COUNT(*) FROM channels;").unwrap()
		.query_map([], |row| row.get(0)).unwrap()
		.flatten()
		.collect();
	db_result[0]
}

#[test]
fn remove_test() {
	ensure_new_database();

	let mut feed = launch_miniserve("./assets/sample1.rss", None);
	let db = Connection::open("corkdb").unwrap();

	run_cork(&["add", "http://localhost:8080"]);
	run_cork(&["remove", "http://localhost:8080"]);
	assert_eq!(get_channel_count(&db), 0);

	run_cork(&["add", "http://localhost:8080"]);
	run_cork(&["remove", "http://another_url"]);
	assert_eq!(get_channel_count(&db), 1);

	run_cork(&["remove", "localhost:8080"]);
	assert_eq!(get_channel_count(&db), 0);

	feed.kill().unwrap();
}
