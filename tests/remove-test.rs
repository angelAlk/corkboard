use rusqlite::Connection;
mod utils;
use utils::*;

#[test]
fn remove_test() {
	ensure_new_database();

	let mut feed = Miniserve::launch("./assets/sample1.rss", None);
	let db = Connection::open("corkdb").unwrap();

	run_cork(&["add", "http://localhost:8080"]);
	run_cork(&["remove", "http://localhost:8080"]);
	assert_eq!(count_channels(&db), 0);

	run_cork(&["add", "http://localhost:8080"]);
	run_cork(&["remove", "http://another_url"]);
	assert_eq!(count_channels(&db), 1);

	run_cork(&["remove", "localhost:8080"]);
	assert_eq!(count_channels(&db), 0);

	feed.kill();
}
