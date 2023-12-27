use std::str::from_utf8;

mod utils;
use utils::*;

//Remember:
//
//Quickmark rules:
//1. If __new__ is run then all marks are reset (using publishing order).
//	(Only unmarked items are considered).
//2. __add__ and __up__ add quickmarks at the end, don't change older ones.
//3. __mark__ deletes the quickmark associated with it.
//4. __remove__ deletes all the quickmarks associated with it, does not affect the rest.
//5. All other commands don't alter the quickmarks.

#[test]
fn test_quickmarks() {
	//A bit of a barebones test to ensure the basics of quickmarks make sense
	ensure_new_database();

	let _feed = Miniserve::launch("./assets/sample2.rss", None);
	assert!(run_cork(&["add", "localhost:8080"]).status.success());

	let mark_result = run_cork(&["mark", "1"]);
	assert!(mark_result.status.success());

	run_cork(&["new"]);
	let mark_result = run_cork(&["mark", "1"]);
	assert!(mark_result.status.success());

	//Marked two so now a single entry must remain
	let last_result = run_cork(&["new"]);
	assert!(last_result.status.success());
	let last_output = from_utf8(&last_result.stdout).expect("Could not read program output");

	//There is still an entry and a mark with a 1 (since we reset the entries)
	assert!(last_output.contains("1"));
	//We know the feed does not contain 2, so the 2 must come from the quickmark
	println!("{last_output}");
	assert!(!last_output.contains("2"));
}
