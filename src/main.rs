//! RSS client
//!
//! Update feeds and keep track of which articles have been read
//! (without being too intrusive with what hasn't been)
//!
//! up <- update rss feeds in the db, show new items
//! new <- show items not yet read
//! mark <item-id> <- mark an item as read
//! add <url> <- add a new rss feed
//! remove <id> <- remove a feed

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
	rss::{Channel, Item}
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
		_ => Err(anyhow::anyhow!("Could not match the operation passed"))
	}?;

	Ok(())
}

///Adds a feed to the database
fn add(database: &Database, url: &str) -> Result<()> {
	let xml_feed = reqwest::blocking::get(url)
		.with_context(|| format!("Network request to feed failed for: {}", url))?
		.text()
		.with_context(|| "Could not turn feed into a string")?;

	let mut channel = xml_handler::xml_to_rss(&xml_feed)
		.with_context(|| "Could not process xml")?;

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

//Update all rss feeds and return the items that were not previously in the database
//fn up(database: &Database) -> Result<Vec<Item>> {
//
//	let client = Client::new();
//	let mut new_items: Vec<Item> = Vec::new();
//
//	for c in database.all_channels()? {
//		let xml_feed = client.get(&c.link)
//			.send()?
//			.text()?;
//		let feed = xml_handler::xml_to_rss(&xml_feed)?;
//
//		//not great
//		if let Some(their_date) = feed.last_build_date {
//			if let Some(our_date) = c.last_build_date {
//				if their_date <= our_date {
//					continue ;
//				}
//			}
//		};
//
//		let items_in_db = database.get_items(&c)?;
//		let mut new_items_in_channel:Vec<Item> = feed.items
//			.into_iter()
//			.filter(|i| !items_in_db.contains(i))
//			.collect();
//
//		database.add_items(&c, &new_items_in_channel)?;
//
//		new_items.append(&mut new_items_in_channel);
//	};
//
//	Ok(new_items)
//}
