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

use std::env::args;
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
	let database = Database::setup("corkdb")?;

	match op {
		Operation::Add(url) => add(&database, &url),
		Operation::Up => up(&database),
		Operation::Feeds => feeds(&database),
		Operation::New => new(&database),
		Operation::Mark(hash) => mark(&database, &hash),
		Operation::Remove(feed_url) => remove(&database, &feed_url)
		//,_ => Err(anyhow::anyhow!("Could not match the operation passed"))
	}?;

	Ok(())
}

///Add a feed and all of it's items into the database
fn add(database: &Database, url: &str) -> Result<()> {
	let xml_feed = reqwest::blocking::get(url)
		.with_context(|| format!("Network request to feed failed for: {}", url))?
		.text()
		.with_context(|| "Could not turn feed into a string")?;

	let mut channel = xml_to_rss(&xml_feed)
		.with_context(|| "Could not process xml")?;

	//TODO: should this line be here?
	//Should we be using the channels announced url or the one we
	//were originally passed ?
	channel.link = String::from(url);

	if let None = channel.last_build_date {
		channel.last_build_date = Some(chrono::Utc::now())
	}

	database.add_channel(&channel)
		.with_context(|| "Failed to add channel to database")?;

	//could and should I "unroll" the changes ?
	database.add_items(&channel, &channel.items)
		.with_context(|| "Failed to add items")?;

	Ok(())
} 

///Get updates from all rss feeds, display the items that are new in the database
//TODO: Add message if nothing is new
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

		if let Err(_) = database.add_items(&c, &new_items) {
			eprintln!("Could not insert new items into database");
			continue ;
		}

		println!("Updates from {}", c.link);
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
	let items = database.all_unmarked_items().context("Could not get items from database")?;
	for i in items {
		println!("{}", i);
	}
	Ok(())
}

///Mark an item in the database as read.
fn mark(database: &Database, hash_string:&str) -> Result<()> {
	database.mark_as_read(hash_string, true)
		.context("Could not mark the article")
}

//Remove feed and it's items from the database
fn remove(database: &Database, url:&str) -> Result<()> {
	database.remove_channel(url)
		.context("Removing for channel failed")
}
