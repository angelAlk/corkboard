//! RSS client
//!
//! Update feeds and keep track of which articles have been read
//! (without being too intrusive with what hasn't been)
//!
//! ```text
//! add <url>		<- Add a new RSS feed to the app.
//! up				<- Update all feeds, show the new articles.
//! mark <item>		<- Mark article/item as read.
//! new				<- Show all articles that haven't been marked.
//! feeds			<- List all the feeds in the app.
//! remove <url>	<- Remove a feed from the app.
//! ```

mod cli;
mod db;
mod request_handler;
mod rss;
mod xml_handler;

use anyhow::{Context, Result};

use std::{
	env::{self, args},
	iter::zip,
	fs,
	path::{Path, PathBuf}
};

use crate::{
	cli::Operation,
	db::Database,
	request_handler::{concurrent_requests, request_feed},
	rss::Item
};

fn main() -> Result<()> {
	let arguments = args().collect();
	let op = cli::parse_arguments(arguments)?;
	run_operation(op)?;
	Ok(())
}

///Set up database and run operation defined by op
fn run_operation(op: Operation) -> Result<()> {
	let database_path = find_database()?;
	let database = Database::setup(database_path)?;

	match op {
		Operation::Add(url) => add(&database, &url),
		Operation::Up => up(&database),
		Operation::Feeds => feeds(&database),
		Operation::New => new(&database),
		Operation::Mark(positions) => mark_relative(&database, &positions),
		Operation::MarkHash(hashes) => mark(&database, &hashes),
		Operation::Remove(feed_url) => remove(&database, &feed_url),
		Operation::Help => print_help()
		//,_ => Err(anyhow::anyhow!("Could not match the operation passed"))
	}?;

	Ok(())
}

///Determine where the corkboard database is or should be, return the path
///
///The database (that is just an sqlite file) may be on:
///  0. If $CORKDB_TEST is set and true, then we just use "corkdb"
///  1.	A custom path, defined by an environment variable $CORKDB,
///    (full path including database name)
///  2. In the XDG directory for program data: $XDG_DATA_HOME
///  3. The default value for $XDG_DATA_HOME: $HOME/.local/share
///  4. If all else fails, again use "./corkdb"
fn find_database() -> Result<PathBuf> {
	//0, testing environment
	if let Ok(are_we_testing) = env::var("CORKDB_TEST") {
		if &are_we_testing == "true" {
			return Ok(Path::new("./corkdb").to_path_buf());
		}
	}

	//1, custom path set with $CORKDB
	if let Ok(custom_path) = env::var("CORKDB") {
		return Ok(Path::new(&custom_path).to_path_buf());
	}

	//2, checking $XDG_DATA_HOME
	if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
		let xdg_path = Path::new(&xdg_data_home);
		if xdg_path.exists() {
			let corkboard_dir = xdg_path.join("./corkboard");
			if !corkboard_dir.exists() {
				fs::create_dir(corkboard_dir.clone())
					.with_context(|| format!("Could not create directory at {corkboard_dir:?}"))?;
			}
			return Ok(corkboard_dir.join("./corkdb").to_path_buf());
		}
	}

	//3, $HOME/.local/share
	if let Ok(home_path) = env::var("HOME") {
		let xdg_default_path = Path::new(&home_path).join(Path::new("./.local/share"));

		if xdg_default_path.exists() {
			let corkboard_dir = xdg_default_path.join("./corkboard");
			if !corkboard_dir.exists() {
				fs::create_dir(corkboard_dir.clone())
					.with_context(|| format!("Could not create directory at {corkboard_dir:?}"))?;
			}
			return Ok(corkboard_dir.join("./corkdb").to_path_buf());
		}
	}

	//4, basic behaviour
	return Ok(Path::new("./corkdb").to_path_buf());
}

///Add a feed and all of it's items into the database
///
///Since we only make 3 petitions these are done sequentially
fn add(database: &Database, url: &str) -> Result<()> {
	//Handling possibly incomplete urls passed by user.
	//The user url is preferred to an https base and https is preferred to an http one.
	let mut working_link: String = String::from(url);
	let mut channel =
		request_feed(&working_link)
		.or_else(|_| {
			working_link = format!("{}{}", "http://", url);
			request_feed(&working_link)})
		.or_else(|_| {
			working_link = format!("{}{}", "https://", url);
			request_feed(&working_link) })?;

	//We keep the users original url and only change it if we had to use another protocol to find the feed
	channel.link = working_link;

	if let None = channel.last_build_date {	
		channel.last_build_date = Some(chrono::Utc::now())
	}

	database.add_channel(&channel)
		.with_context(|| "Failed to add channel to database")?;

	//could and should I "unroll" the changes ?
	database.add_items(&channel, &channel.items)
		.with_context(|| "Failed to add items")?;

	database.generate_quickmarks(&channel.items)
		.context("Failed to create quickmarks for the items")?;

	Ok(())
} 

///Get updates from all rss feeds, display the items that are new in the database
fn up(database: &Database) -> Result<()> {
	let channels = database.all_channels_with_items()
		.context("Failed to get all the channels for the update.")?;

	//here we would like to ignore the error if one fails,
	//we'll post something about a failure but keep running with the
	//rest of the channels
	
	let channel_links:Vec<_> = channels.iter().map(|c| c.link.as_str()).collect();
	
	let updated_feeds = concurrent_requests(&channel_links[..]);

	for (old_feed, new_feed) in zip(channels, updated_feeds) {
		if let Err(e) = new_feed {
			eprintln!("{e}");
			continue ;
		}
		let new_feed = new_feed.unwrap();

		if let (Some(their_date), Some(our_date)) = (new_feed.last_build_date, old_feed.last_build_date) {
			if our_date >= their_date {
				continue ;
			}
		}

		let new_items:Vec<Item> = new_feed.items.into_iter()
			.filter(|i| !old_feed.items.contains(i))
			.collect();

		if new_items.len() == 0 {
			continue ;
		}

		if let Err(_) = database.add_items(&old_feed, &new_items) {
			eprintln!("Could not insert new items into database");
			continue ;
		}
		//I'm unsure if we should update quickmarks on up since we aren't displaying them ever ?
		database.generate_quickmarks(&new_items)?;

		println!("Updates from {}", old_feed.link);
		for i in new_items {
			println!("\t {} at {}", i.title_or_description, i.link.unwrap_or(String::from("<NO LINK>")));
		}
	}

	Ok(())
}

///List all feeds in the database.
fn feeds(database: &Database) -> Result<()> {
	let channels = database.all_channels()
		.context("Could not get channels from the database")?;
	if channels.len() == 0 {
		println!("No RSS feeds in the database");
	} else {
		for c in channels {
			println!("{}", c.link);
		}
	}
	Ok(())
}

///Show all the items not yet marked (read by the user)
fn new(database: &Database) -> Result<()> {
	database.reset_quickmarks()
		.context("Failed to write to database, reset quickmarks")?;

	let mut items:Vec<(Item, i32)> = database.all_unmarked_items_with_quickmarks()
		.context("Could not get items from the database")?;
	items.sort_by_key(|t| t.1);

	for (item, position) in items {
		//FIX: move displaying to cli module
		println!("{} -> [{}] {}",
				 position,
				 item.link.as_ref().unwrap_or(&String::from("No link")),
				 item.title_or_description);
	}

	Ok(())
}

///Mark an item in the database as read.
fn mark(database: &Database, hash_string:&[String]) -> Result<()> {
	for hash in hash_string {
		database.mark_as_read(hash, true)
			.context("Could not mark the article")?;
		database.remove_quickmark(hash)
			.context("Could not delete quickmark associated with article")?;
	}
	Ok(())
}

///Mark an item in the database as read when given it's position
///as printed by the _new_ command.
fn mark_relative(database: &Database, positions:&[usize]) -> Result<()> {
	//This is one of the places where I am unsure about how to manage errors
	//I belive that for myself the easiest option is to just ignore
	//failed mark attempts, but I can see how perhaps a user would prefer
	//that the whole command fails.
	//
	//For now I am chosing to ignore failed marks and keep going.
	//I am printing a message still.

	for p in positions {
		match database.mark_as_read_with_quickmark(*p) {
			Ok(_) => { println!("Marked item {p}"); },
			Err(e) => { println!("Could not mark {p} due to {e}. Moving on"); }
		};
	}

	Ok(())
}

///Remove feed and it's items from the database
fn remove(database: &Database, url:&str) -> Result<()> {
	database.remove_channel(url)
		.context("Removing for channel failed")
}

///Print the help message for the program
fn print_help() -> Result<()> {
	let msg: &str = "\
Minimal RSS client

usage: corkboard <command>

Commands:
  add <url>             Subscribe to a feed with url <url>.
  up                    Update all feeds then display all the items/posts that were added.
  feeds                 List all subscribed feeds.
  new                   Show all items not marked as read (does not update channels).
  mark <item-hash+>     Mark an item as read.
  remove <url>          Unsuscribe from a feed and delete all of it's items from the database.
  help                  Show this help message.
";
	println!("{msg}");
	Ok(())
}

