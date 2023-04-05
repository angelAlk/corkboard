use rusqlite::Connection;
mod utils;
use utils::*;

#[test]
fn adding_without_https_in_url() {
	//when a user calls cargo add <url>
	//we want to treat a.domain.com/feed and https://a.domain.com/feed as the same

	ensure_new_database();
	let db = Connection::open("corkdb").unwrap();

	let mut feed = Miniserve::launch("./assets/sample1.rss", None);
	assert!(run_cork(&["add", "localhost:8080"]).status.success());
	feed.kill();

	assert_eq!(1, count_channels(&db));

	feed = Miniserve::launch("./assets/sample1.rss", None);
	//Add should fail since http://localhost and localhost should be considered the same channel.
	assert!(!run_cork(&["add", "http://localhost:8080"]).status.success());
 
	assert_eq!(1, count_channels(&db));
}
