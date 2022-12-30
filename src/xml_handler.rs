//! Parsing the RSS XML into structs we can handle

use anyhow;
use chrono::{DateTime, offset::FixedOffset};
use roxmltree::{Node};
use std::{fmt, error};
use crate::rss::{Channel, Item};

#[derive(Debug)]
enum XmlError {
	RSSTagIsNotRoot,
	NoChannelTag,
	NoTitle,
	NoLink,
	NoDesc
}
impl fmt::Display for XmlError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let err_string = match self {
			XmlError::RSSTagIsNotRoot => "rss tag is not at the top of the xml passed",
			XmlError::NoChannelTag => "The channel tag was not found in the xml passed",
			XmlError::NoTitle => "The title for the channel is not present in the xml passed",
			XmlError::NoLink => "The link for the channel is not present in the xml passed",
			_ => "xml error"
		};

		write!(f, "{}", err_string)
	}
}
impl error::Error for XmlError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		None
	}
}

/// Tries to find a direct child of the parent tag with the name _name_
fn get_named_child<'a>(parent: &Node<'a, 'a>, name: &str) -> Option<Node<'a, 'a>> {
	//Maybe we should be normalizing the strings before comparing them
	parent
		.children()
		.find(|c| c.tag_name().name() == name)
}

fn get_text_from_child(parent: &Node, name: &str) -> Option<String> {
	let borrowed = get_named_child(parent, name)?.text()?;
	Some(String::from(borrowed))
}

fn process_item(item_tag: &Node) -> Option<Item> {
	let title_or_description:String = get_text_from_child(item_tag, "title")
		.or(get_text_from_child(item_tag, "description"))?;

	let link:Option<_> = get_text_from_child(item_tag, "link");

	let pub_date:Option<_> = get_text_from_child(item_tag, "pubDate")
		.and_then(|date_s| DateTime::parse_from_rfc2822(&date_s).ok());

	Some(Item::new(title_or_description, link, pub_date))
}

/// Turns an xml string into a Channel struct, if the xml is misformed
/// then an error is returned.
pub fn xml_to_rss (xml_source: &str) -> anyhow::Result<Channel> {
	let xml_tree = roxmltree::Document::parse(&xml_source)?;
	let root = xml_tree.root_element();//RSS element
	
	if root.tag_name().name() != "rss" {
		Err(XmlError::RSSTagIsNotRoot)?
	}
	let channel_tag = get_named_child(&root, "channel")
		.ok_or(XmlError::NoChannelTag)?;

	let title = get_text_from_child(&channel_tag, "title")
		.ok_or(XmlError::NoTitle)?;
	let link = get_text_from_child(&channel_tag, "link")
		.ok_or(XmlError::NoLink)?;
	let description = get_text_from_child(&channel_tag, "description")
		.ok_or(XmlError::NoDesc)?;

	let last_build_date:Option<DateTime<FixedOffset>> = get_text_from_child(&channel_tag, "lastBuildDate")
		.and_then(|date_s| DateTime::parse_from_rfc2822(&date_s).ok());

	let items:Vec<Item> = channel_tag .children()
		.filter(|c| c.tag_name().name() == "item")
		.map(|i| process_item(&i))
		.filter(|i| i.is_some())
		.map(|i| i.unwrap())
		.collect();

	Ok(Channel {
		title,
		link,
		description,
		last_build_date,
		items
	})
}

