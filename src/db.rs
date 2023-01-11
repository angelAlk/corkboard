//! Interface into a sqlite database

use anyhow::Result;
use std::str::FromStr;
use rusqlite::{Connection, params};

use crate::rss::{Channel, Item};

///Encapsulates a connection to the sqlite db
pub struct Database {
	///The rusqlite connection to the database
	db: Connection
}
impl Database {
	///Initialize or connect to a sqlite database
	pub fn setup(db_path: &str) -> Result<Self> {
		let db = Connection::open(db_path)?;

		//Channel entries
		db.execute(
			"CREATE TABLE IF NOT EXISTS channels (
				id INTEGER PRIMARY KEY AUTOINCREMENT,
				title VARCHAR(256),
				link VARCHAR(256) UNIQUE,
				description TEXT,
				last_build_date VARCHAR
			);", [])?;

		//Item entries
		db.execute(
			"CREATE TABLE IF NOT EXISTS items (
				hash VARCHAR PRIMARY KEY,
				title_or_desc VARCHAR(256) NOT NULL,
				url VARCHAR(256),
				pub_date VARCHAR,
				read BOOLEAN NOT NULL,
				channel INTEGER NOT NULL,
				FOREIGN KEY(channel) REFERENCES channels(id)
			);", [])?;

		Ok(Self {db})
	}

	///Get all the channels from the database (without their respective items)
	pub fn all_channels(&self) -> Result<Vec<Channel>> {
		let mut statement = self.db.prepare(
			"SELECT title, link, description, last_build_date
			FROM channels;"
		)?;
		let channels = statement.query_map([], |row| {
			Ok(Channel {
				title: row.get(0)?,
				link: row.get(1)?,
				description: row.get(2)?,
				last_build_date: row.get(3)?,
				items: Vec::new()
			})
		})?;

		Ok(channels.flatten().collect())
	}

	///Get all the channels from the database (with their items)
	pub fn all_channels_with_items(&self) -> Result<Vec<Channel>> {
		let mut channels = self.all_channels()?;

		for c in channels.iter_mut() {
			c.items = self.get_items(&c)?;
		}

		Ok(channels)
	}

	///Returns the items in the database that belong to a channel.
	pub fn get_items(&self, channel: &Channel) -> Result<Vec<Item>> {
		let mut statement = self.db.prepare(
			"SELECT hash, title_or_desc, url, pub_date, read
			FROM items LEFT JOIN channels ON items.channel == channels.id
			WHERE link = (?);"
		)?;

		let items = statement.query_map([&channel.link], |row| {
			Ok(Item {
				//canzer
				title_or_description_hash: u64::from_str(&row.get::<usize, String>(0)?).unwrap(),
				title_or_description: row.get(1)?,
				link: row.get(2)?,
				pub_date: row.get(3)?,
				read: row.get(4)?
			})
		})?;

		Ok(items.flatten().collect())
	}

	//Add a new channel into db
	pub fn add_channel(&self, channel: &Channel) -> Result<()> {
		let mut statement = self.db.prepare(
			"INSERT INTO channels (title, link, description, last_build_date)
			VALUES (?,?,?,?);"
		)?;

		statement.execute(rusqlite::params![
			channel.title,
			channel.link,
			channel.description,
			channel.last_build_date
		])?;

		Ok(())
	}

	//Adds new items to the database, associates them with the channel passed.
	//Note that if the items have the same hash as another in the database
	//then the insertion is ignored.
	pub fn add_items(&self, channel: &Channel, items: &[Item]) -> Result<()> {
		let channel_id:u64 = self.db.prepare("SELECT id FROM channels WHERE link = (?);")?
			.query_row([&channel.link], |row| {row.get(0)})?;

		let mut statement = self.db.prepare(
			"INSERT OR IGNORE
			INTO items (hash, title_or_desc, url, pub_date, read, channel)
			VALUES (?, ?, ?, ?, ?, ?);"
		)?;

		for i in items {
			statement.execute(rusqlite::params![
				i.title_or_description_hash.to_string(),
				i.title_or_description,
				i.link,
				i.pub_date,
				i.read,
				channel_id
			])?;
		}

		Ok(())
	}
}
