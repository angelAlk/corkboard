mod utils;
use utils::*;

use rusqlite::Connection;

/// Basically have two feeds that
/// have the same title (say because they were auto-generated)
/// but that point to different resources, well these feeds
/// should be able to coexist on the db
#[test]
fn titles_clash() {
	ensure_new_database();

	let mut feed_a = Miniserve::launch("./assets/clash-a.rss", None);
	let out = run_cork(&["add", "localhost:8080"]);
	assert!(out.status.success());
	feed_a.kill();

	let mut feed_b = Miniserve::launch("./assets/clash-b.rss", Some(&["--port", "9090"]));
	let out = run_cork(&["add", "localhost:9090"]);
	println!("{:?}", out);
	assert!(out.status.success());
	feed_b.kill();

	//open database, just ensure that there are two items, one
	//from each clash feed
	let db = Connection::open("corkdb").unwrap();
	let items:Vec<i64> = db.prepare("SELECT * FROM items").expect("DB fail")
		.query_map([], |_row| Ok(1)).expect("DB item fails")
		.map(|x| x.unwrap())
		.collect();

	assert_eq!(items.len(), 2);
}
