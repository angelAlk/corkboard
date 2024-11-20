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

pub mod rss;
mod xml_handler;
mod db;
mod cli;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use url::Url;

use std::{
	env::{self, args},
	fs,
	path::{Path, PathBuf}
};

use crate::{
	cli::Operation,
	db::Database,
	rss::{Channel, Item},
	xml_handler::xml_to_rss
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
		Operation::MarkAll => mark_all(&database),
		Operation::MarkHash(hashes) => mark(&database, &hashes),
		Operation::Remove(feed_url) => remove(&database, &feed_url),
		Operation::Help => print_help()
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

///Request a feed with url parsing and error handling
fn request_feed(feed_source: &str) -> Result<String> {
	let parsed_source = Url::parse(feed_source)?;

	reqwest::blocking::get(parsed_source)
		.with_context(|| format!("Network request to feed failed for: {}", feed_source))?
		.text()
		.with_context(|| "Could not turn feed into a string")
}

///Add a feed and all of it's items into the database
fn add(database: &Database, url: &str) -> Result<()> {
	//url normalization
	//If protocol defined by user, then use it.
	//If not, try https, then http
	let (working_link, xml_feed) = match url.find("http") {
		Some(idx) if idx==0 => (String::from(url), request_feed(url)?),
		_ => {
			let https_link = format!("https://{url}");
			let http_link = format!("http://{url}");

			match request_feed(&https_link) {
				Ok(https_feed) => (https_link, https_feed),
				Err(_) =>         ( http_link.clone(), request_feed(&http_link)? )
			}
		}
	};

	let mut channel = xml_to_rss(&xml_feed)
		.with_context(|| "Could not process xml")?;

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
	let client = Client::new();

	let channels = database.all_channels_with_items()
		.context("Failed to get all the channels for the update.")?;

	//here we would like to ignore the error if one fails,
	//we'll post something about a failure but keep running with the
	//rest of the channels
	
	for c in channels {
		let Ok(feed) = get_feed(&c.link, &client) else {
			eprintln!("Failed to reach or parse: {}", c.link);
			continue;
		};

		if let (Some(their_date), Some(our_date)) = (feed.last_build_date, c.last_build_date) {
			if our_date >= their_date {
				continue ;
			}
		}

		let new_items:Vec<Item> = feed.items.into_iter()
			.filter(|i| !c.items.contains(i))
			.collect();

		if new_items.len() == 0 {
			continue ;
		}

		if let Err(db_e) = database.add_items(&c, &new_items) {
			eprintln!("Could not insert new items into database: {db_e}");
			continue ;
		}
		//I'm unsure if we should update quickmarks on up since we aren't displaying them ever ?
		database.generate_quickmarks(&new_items)?;

		println!("Updates from \"{}\" ({})", c.title, c.link);
		for i in new_items {
			println!("\t {} at {}", i.title_or_description, i.link.unwrap_or(String::from("<NO LINK>")));
		}
	}
	Ok(())
}

fn get_feed(url: &str, req_client: &Client) -> Result<Channel> {
	let xml_feed = req_client.get(url)
		.send().with_context(|| format!("Request to:{} failed", url))?
		.text().with_context(|| format!("Response from:{}, could not be turned into text", url))?;
	xml_to_rss(&xml_feed).context("Could not parse XML into RSS")
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

///Mark all items as read.
fn mark_all(database: &Database) -> Result<()> {
	let items = database.all_unmarked_items()
		.context("Could not get open items from database")?;

	for item in items {
		database.mark_as_read(&item.title_or_description_hash, true)
			.context("Coulld not mark the item")?;
	}

	database.reset_quickmarks()
		.context("Failed to reset quickmarks")?;

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
  mark <number>         Mark an item at position <number> as read. Positions come from corkboard new.
  remove <url>          Unsuscribe from a feed and delete all of it's items from the database.
  help                  Show this help message.
";
	println!("{msg}");
	Ok(())
}

