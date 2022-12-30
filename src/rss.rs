//! RSS is a protocol based on XML files, the tree structure is:
//!
//! ```
//! rss
//!	-channel
//! --title
//! --link
//! --description
//! --lastBuildDate
//! --item <--- at least title or description present
//! ---title
//! ---link
//! ---description
//! ---guid <--- useless?
//! ---pubdate
//! ```

use chrono::{
	offset::FixedOffset,
	DateTime
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};

///Represents a single item in an RSS channel.
///
///It's only ensured that either a description tag or a title tag
///will be present in it.
#[derive(Debug)]
pub struct Item {
	///The title of the item or in it's absence the description
	pub title_or_description: String,
	///hash of the title_or_description key. Will act as the primary
	///key of Item in our DB (alongside the channel id)
	pub title_or_description_hash:u64,
	///URL to the item,blog post or entry
	pub link: Option<String>,
	///Date that the item was published
	pub pub_date: Option<DateTime<FixedOffset>>,
	///Whether the user has read or not this item
	pub read: bool
}
impl Item {

	///Create a new not yet read item.
	///
	///We are using the default hasher for now but should probably move to fixed one (say sha256).
	pub fn new(title_or_desc: String, link: Option<String>, pub_date: Option<DateTime<FixedOffset>>) -> Self {
		let mut s = DefaultHasher::new();
		title_or_desc.hash(&mut s);
		let hash = s.finish();

		Self {
			title_or_description: title_or_desc,
			title_or_description_hash: hash,
			link,
			pub_date,
			read: false
		}
	}
}


///RSS channel (which should correspond to a whole website or blog).
#[derive(Debug)]
pub struct Channel {
	///Name of the channel
	pub title:String,
	///URL to the main page of the channel (declared in channel tag)
	pub link:String,//don't handle as urls ?
	///Description of the channel itself, will usually be about the blog
	///or site being followed
	pub description:String,
	///Date that the channel last changed, if the rss one is the same or older than
	///then one on our DB then we don't need to do anything.
	pub last_build_date: Option<DateTime<FixedOffset>>,
	///Items present in the channel
	pub items: Vec<Item>
}
