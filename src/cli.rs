//! Handles input and output for the terminal interface

use anyhow::Result;

///The actions available to the user of the program.
pub enum Operation {
	///Add a new channel to the database.
	Add(String),
	Up,
	Feeds
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
		_ => Err(ParseErr::NotACommand.into())
	}
}

// up <- update rss feeds in the db, show new items
// new <- show items not yet read
// mark <item-id> <- mark an item as read
// add <url> <- add a new rss feed
// remove <id> <- remove a feed
