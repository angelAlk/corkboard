use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	str::from_utf8
};

mod utils;
use utils::*;

//Hashes a string and formats it in the same way as the constructor in the rss module.
fn hash_string(s:&str) -> String {
	let mut h = DefaultHasher::new();
	s.hash(&mut h);
	let hash:u64 = h.finish();
	format!("{:016x}", hash)
}

#[test]
fn new_then_mark() {
	ensure_new_database();

	//start the source feed and add it to the program
	let mut feed = Miniserve::launch("./assets/sample3.rss", None);
	let add_correct = run_cork(&["add", "http://localhost:8080"]).status.success();
	assert!(add_correct);

	//Get the new items
	let new_output = run_cork(&["new"]);
	//sample3.rss has two articles with titles 'azz' and 'bzz' we want to see that both show up
	//as new articles.
	let articles:Vec<&str> = from_utf8(&new_output.stdout).expect("Could not read output as string")
		.split("\n")
		.filter(|line| line.contains("azz") || line.contains("bzz"))
		.collect();
	assert_eq!(articles.len(), 2);

	//we mark the article with title 'azz' as read.
	//Here we are implicitly testing that if the title is present
	//then it's the only input to the hash.
	let mark_output = run_cork(&["mark", &hash_string("azz")]).status.success();
	assert!(mark_output);

	//Get the items that we haven't read, we expect to only get 'bzz'
	let second_new_output = run_cork(&["new"]);
	let second_articles = from_utf8(&second_new_output.stdout).expect("Could not read output as string");
	assert!(second_articles.contains("bzz"));
	assert!(!second_articles.contains("azz"));

	//kill the feed source
	feed.kill();
}
