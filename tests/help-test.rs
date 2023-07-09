mod utils;
use utils::run_cork;


///Test that the help commands exist and that they print
///a string.
///How should we test the contents of the help string ?
#[test]
fn help_test() {
	let mut outputs:Vec<String> = vec!["help", "-h", "--help"]
		.iter()
		.map(|subcommand| run_cork(&[subcommand]))
		.map(|output| output.stdout)
		.map(String::from_utf8)
		.map(|res| res.expect(&format!("Error while calling help with a variation")))
		.collect();

	let base_output = outputs.remove(0);
	let shorthand_output = outputs.remove(0);
	let gnu_style_output = outputs.remove(0);

	assert_eq!(base_output, shorthand_output);
	assert_eq!(base_output, gnu_style_output);

	assert!(base_output.len() > 0);
}
