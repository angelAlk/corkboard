mod utils;
use utils::*;

use std::str::from_utf8;

#[test]
fn mark_all_many_feeds() {
	ensure_new_database();

	//Load a feed
	let _miniserve = Miniserve::launch("./assets/sample1.rss", None);
	let did_add_succeed = run_cork(&["add", "http://localhost:8080"]).status.success();
	assert!(did_add_succeed);

	//See that the two items are present in corkboard new.
	let output_of_new = run_cork(&["new"]);
	assert!(output_of_new.status.success());
	let text_output_of_new = from_utf8(&output_of_new.stdout).expect("Could not read corkboard output as string");
	//NOTE: This asserts that corkboard new will always have one item per line.
	assert!(text_output_of_new.trim_end().split("\n").collect::<Vec<&str>>().len() == 2);

	//mark all
	let mark_all_success = run_cork(&["mark", "--all"]).status.success();
	assert!(mark_all_success);

	//see that there are no entries in corkboard new.
	let second_output_of_new = run_cork(&["new"]);
	assert!(second_output_of_new.status.success());
	let second_text_output_of_new = from_utf8(&second_output_of_new.stdout).expect("Could not read corkboard output as string");
	//output should be empty
	assert!(second_text_output_of_new == "");
}
