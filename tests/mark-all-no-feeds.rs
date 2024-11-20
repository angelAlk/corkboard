mod utils;
use utils::*;

#[test]
fn mark_all_no_feeds() {
	//just needs to do one thing. Don't crash.
	ensure_new_database();
	assert!(run_cork(&["mark", "--all"]).status.success());
}
