//! Interface into the corkboard sqlite database
//!
//! We have three tables: _channels_, _items_ and _quickmarks_
//!
//! _channels_ stores the RSS feeds and owns many (or zero) _items_.
//!
//! _items_ store specific entries from a feed, are owned by _channels_.
//!
//! _quickmarks_ holds the mark system (that simplifies the usage of **corkboard new**)
//!
//! The quickmark system works as follows:
//!
//! To make marking an item as read easier we use an
//! assigned number instead of the item's hash.
//! We store those assigned numbers (which are ordered by publishing date)
//! in this table.
//!
//! If we expect quickmarks to be ordered by publishing date then
//! couldn't we just sort by date and use the positions on the vec
//! to know what to mark ? Why have another table ?
//!
//! We want these "quickmarks" to be consistent between runs of the program.
//! As an example, if the user runs:
//!
//! __new__
//!
//! __add__
//!
//! __mark__
//!
//! Then __mark__ could fail or mark as read another different item than
//! the intended one. Because __add__ inserted new items into the publishing order.
//!
//! Quickmark rules:
//! 1. If __new__ is run then all marks are reset (using publishing order).
//! 	(Only unmarked items are considered).
//! 2. __add__ and __up__ add quickmarks at the end, don't change older ones.
//! 3. __mark__ & __mark all__ delete the quickmark associated with it.
//! 4. __remove__ deletes all the quickmarks associated with it, does not affect the rest.
//! 5. All other commands don't alter the quickmarks.

use anyhow::{anyhow, Context, Result};
use rusqlite::{Connection, params};

use std::path::Path;

use crate::rss::{Channel, Item};

///Encapsulates a connection to the sqlite db
pub struct Database {
	///The rusqlite connection to the database
	db: Connection
}
impl Database {
	///Initialize or connect to a sqlite database
	pub fn setup<P: AsRef<Path>>(db_path: P) -> Result<Self> {
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
				FOREIGN KEY(channel) REFERENCES channels(id) ON DELETE CASCADE
			);", [])?;

		//Quickmarks
		db.execute(
			"CREATE TABLE IF NOT EXISTS quickmarks (
				position INTEGER,
				hash VARCHAR,
				FOREIGN KEY(hash) REFERENCES items(hash) ON DELETE CASCADE
			);" ,[])?;

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
				title_or_description_hash: row.get(0)?,
				title_or_description: row.get(1)?,
				link: row.get(2)?,
				pub_date: row.get(3)?,
				read: row.get(4)?
			})
		})?;

		Ok(items.flatten().collect())
	}

	///Add a new channel into db
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

	///Adds new items to the database, associates them with the channel passed.
	///Note that if the items have the same hash as another in the database
	///then the insertion is ignored.
	pub fn add_items(&self, channel: &Channel, items: &[Item]) -> Result<()> {
		let channel_id:u64 = self.db.prepare("SELECT id FROM channels WHERE link = (?);")?
			.query_row([&channel.link], |row| {row.get(0)})?;

		let mut statement = self.db.prepare(
			"INSERT
			INTO items (hash, title_or_desc, url, pub_date, read, channel)
			VALUES (?, ?, ?, ?, ?, ?);"
		)?;

		for i in items {
			statement.execute(rusqlite::params![
				i.title_or_description_hash,
				i.title_or_description,
				i.link,
				i.pub_date,
				i.read,
				channel_id
			]).context(i.title_or_description_hash.clone())?;
		}

		Ok(())
	}

	///Return all the items from the database that have not been read.
	pub fn all_unmarked_items(&self) -> Result<Vec<Item>> {
		let mut statement = self.db.prepare("SELECT hash, title_or_desc, url, pub_date, read
						FROM items
						WHERE read=0;"
		)?;

		let items = statement.query_map([], |row| {
			Ok(Item {
				title_or_description_hash: row.get(0)?,
				title_or_description: row.get(1)?,
				link: row.get(2)?,
				pub_date: row.get(3)?,
				read: row.get(4)?
			})
		})?;

		Ok(items.flatten().collect())
	}

	///Mark item as read given it's hash
	pub fn mark_as_read(&self, hash: &str, read_state:bool) -> Result<()> {
		let mut statement = self.db.prepare(
			"UPDATE items
			SET read=(?)
			WHERE hash=(?);"
		)?;
		let rows_changed = statement.execute(params![isize::from(read_state),hash])?;

		if rows_changed == 1 {
			Ok(())
		} else {
			Err(anyhow!("Expected to change a single row, {} rows changed.", rows_changed))
		}
	}

	///Remove the feed with url feed_url from the database
	pub fn remove_channel(&self, feed_url: &str) -> Result<()> {
		//SUPPOSITION: all feeds that are equal except in protocol are equal in content.
		let mut statement = self.db.prepare(
			"DELETE FROM channels WHERE link=(?1) OR link=(?2) OR link=(?3);"
		)?;

		//BELIEVE rusqlite acting weirdly
		//HACKY workaround, passing the complete urls
		let rows_deleted = statement.execute(params![
			feed_url,
			&format!("http://{}", feed_url),
			&format!("https://{}", feed_url)
		])?;

		//NOTE: how does this work with cascading ? might be a source of bugs.
		if rows_deleted == 1 {
			Ok(())
		} else {
			Err(anyhow!("Expected to delete a single row, {} rows deleted.", rows_deleted))
		}
	}

	//QUICKMARKS---

	///Deletes all the quickmarks in the database and adds marks for all the
	///unmarked items in publishing order.
	pub fn reset_quickmarks(&self) -> Result<()> {
		let mut delete_quickmarks_st = self.db.prepare("DELETE FROM quickmarks;")?;
		delete_quickmarks_st.execute([])?;

		let mut add_quickmark_st = self.db.prepare(
			"INSERT
			INTO quickmarks (position, hash)
			VALUES (?, ?); "
		)?;

		let mut unmarked_items = self.all_unmarked_items()?;
		unmarked_items.sort_by_key(|i| i.pub_date);
		for i in 0..unmarked_items.len() {
			let item = &unmarked_items[i];
			add_quickmark_st.execute(params![i+1, item.title_or_description_hash])?;
		}

		Ok(())
	}

	///Get all the items in the database that have not been read, and with
	///them get their quickmark position.
	pub fn all_unmarked_items_with_quickmarks(&self) -> Result<Vec<(Item, i32)>> {
		let mut get_st = self.db.prepare(
			"SELECT hash, title_or_desc, url, pub_date, read, position
			FROM items INNER JOIN quickmarks USING(hash)
			WHERE read=0;"
		)?;

		let items = get_st.query_map([], |row| {
			Ok( (
				Item {
					title_or_description_hash: row.get(0)?,
					title_or_description: row.get(1)?,
					link: row.get(2)?,
					pub_date: row.get(3)?,
					read: row.get(4)?
				},
				row.get(5)?)
			)
		})?;

		Ok(items.flatten().collect())
	}

	///Creates new quickmarks for the items passed as argument.
	///These marks don't overwrite, nor affect the marks already stored.
	pub fn generate_quickmarks(&self, items: &[Item]) -> Result<()> {
		//first step, find max quickmark
		//then insert a quickmark for each item, counting up from the old_max

		//Should we instead store the max quickmark somewhere ?
		let mut max_quickmark_st = self.db.prepare(
			"SELECT position FROM quickmarks;"
		)?;
		let max_quickmark = max_quickmark_st.query_map([], |row| row.get(0))?
			.flatten()
			.max()
			.unwrap_or(0);

		let mut insert_quickmark_st = self.db.prepare(
			"INSERT into quickmarks (hash, position) VALUES (?, ?);"
		)?;
		let mut position = max_quickmark + 1;
		for i in items {
			insert_quickmark_st.execute(params![i.title_or_description_hash, position])?;
			position += 1;
		}
		Ok(())
	}

	///Deletes only the quickmark of a an item (passing it's title hash)
	pub fn remove_quickmark(&self, hash: &str) -> Result<()> {
		let mut rm_st = self.db.prepare(
			"DELETE FROM quickmarks WHERE hash=?;"
		)?;
		rm_st.execute(params![hash])?;
		Ok(())
	}

	///Marks an item as read when passed it's quickmark.
	///
	///ignores invalid quickmarks.
	pub fn mark_as_read_with_quickmark(&self, mark: usize) -> Result<()> {
		let mut statement = self.db.prepare(
			"UPDATE items
			SET read=TRUE
			WHERE hash=
				(SELECT hash FROM quickmarks WHERE position = (?));"
		)?;
		statement.execute(params![mark])?;

		self.db.prepare("DELETE FROM quickmarks WHERE position = (?);")?
			.execute(params![mark])?;

		Ok(())
	}
}
