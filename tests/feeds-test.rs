mod utils;
use utils::*;

//Need to think of a better way of testing this
#[test]
fn feeds_test() {
	ensure_new_database();

	for i in 0..=12 {
		let mut f = launch_miniserve("./assets/sample1.rss", Some(&["--port", &format!("80{i:02}")]));
		run_cork(&["add", &format!("http://localhost:80{i:02}")]);
		f.kill().unwrap();
	}

	let output = run_cork(&["feeds"]);
	let output = std::str::from_utf8(&output.stdout).expect("Could not read output as string");

	let output_lines = output.lines()
		.collect::<Vec<&str>>()
		.len();

	//we expect to see at least the 13 feeds we added to the database
	assert!(output_lines >= 13);
}
