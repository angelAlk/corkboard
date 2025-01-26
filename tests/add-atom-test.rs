//OBS: If we can succesfully add an Atom feed, then
//we must necessarily be parsing it successfully

use rusqlite::Connection;

mod utils;
use utils::*;

#[test]
fn add_atom_test() {
	// Empty atom feed
	ensure_new_database();
	let mut first_sample = Miniserve::launch("./assets/atom1.rss", None);
	let first_output = run_cork(&["add", "http://localhost:8080"]);
	first_sample.kill();
	assert!(first_output.status.success());
	let db = Connection::open("corkdb").unwrap();
	assert_eq!(count_channels(&db), 1);
	assert_eq!(count_items(&db), 0);

	// An atom feed with two entries
	ensure_new_database();
	let mut second_sample = Miniserve::launch("./assets/atom2.rss", None);
	let second_output = run_cork(&["add", "http://localhost:8080"]);
	second_sample.kill();
	assert!(second_output.status.success());
	let db = Connection::open("corkdb").unwrap();
	assert_eq!(count_channels(&db), 1);
	assert_eq!(count_items(&db), 2);
}
