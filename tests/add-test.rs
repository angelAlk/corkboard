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

//#[test]
//fn adding_the_same_feed_twice() {
//	todo!();
//}

//#[test]
//fn adding_without_https_in_url() {
//	//when a user calls cargo add <url>
//	//we want to treat a.domain.com/feed and https://a.domain.com/feed as the same
//
//	ensure_new_database();
//	let db = Connection::open("corkdb").unwrap();
//
//	let mut feed = Miniserve::launch("./assets/sample1.rss", None);
//	run_cork(&["add", "localhost:8080"]);
//	feed.kill();
//
//	assert_eq!(1, count_channels(&db));
//
//	feed = Miniserve::launch("./assets/sample1.rss", None);
//	run_cork(&["add", "https://localhost:8080"]);
//	feed.kill();
//
//	assert_eq!(1, count_channels(&db));
//}
