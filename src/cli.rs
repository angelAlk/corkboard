//! Handles input and output for the terminal interface

use anyhow::Result;

///The actions available to the user of the program.
pub enum Operation {
	///Add a new channel to the database.
	Add(String),
	///Check the updates in the feeds
	Up,
	///List all the feeds in the database
	Feeds,
	///Show which feeds are new
	New,
	///Mark an item as read
	Mark(Vec<usize>),
	///Mark an item as read, using it's hash
	MarkHash(Vec<String>),
	///Remove a feed from the db
	Remove(String),
	///Print the help message for the program
	Help
}

#[derive(Debug)]
///Errors ocurring while parsing arguments
pub enum ParseErr {
	NoArguments,
	NotACommand
}
impl std::fmt::Display for ParseErr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ParseErr::NoArguments => write!(f, "No arguments passed to the program"),
			ParseErr::NotACommand => write!(f, "The argument passed is not a valid command")
		}
	}
}
impl std::error::Error for ParseErr {}

///Turn the arguments passed to the program into a usable struct
pub fn parse_arguments(string_args:Vec<String>) -> Result<Operation> {
	anyhow::ensure!(string_args.len() > 1, ParseErr::NoArguments);

	match string_args[1].as_str() {
		"add" if string_args.len() >= 3 => Ok(Operation::Add(string_args[2].clone())),

		"up" => Ok(Operation::Up),

		"feeds" => Ok(Operation::Feeds),

		"new" => Ok(Operation::New),

		"mark" if string_args.len() >= 3 => {
			let positions: Result<Vec<_>, _>= string_args[2..].iter()
				.map(|pos| str::parse::<usize>(&pos))
				.collect();
			Ok(Operation::Mark(positions?))
		},

		"markhash" if string_args.len() > 2 => {
			let hashes = string_args[2..].to_vec();
			Ok(Operation::MarkHash(hashes))
		},

		"remove" if string_args.len() >= 3 => Ok(Operation::Remove(string_args[2].clone())),

		"help" | "-h" | "--help" => Ok(Operation::Help),

		_ => Err(ParseErr::NotACommand.into())
	}
}
