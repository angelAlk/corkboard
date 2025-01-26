//! Parsing the RSS XML into structs we can handle

use chrono::{DateTime, offset::Utc};
use roxmltree::Node;
use std::{fmt, error};
use crate::rss::{Channel, Item};

#[derive(Debug)]
pub enum XmlError {
	UnknownFormat,
	ParserFailed,
	NoChannelTag,
	NoTitle,
	NoLink
}
impl fmt::Display for XmlError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let err_string = match self {
			XmlError::UnknownFormat => "Format of the XML passed is neither Atom nor RSS",
			XmlError::ParserFailed => "Could not parse XML",
			XmlError::NoChannelTag => "The channel tag was not found in the xml passed",
			XmlError::NoTitle => "The title for the channel is not present in the xml passed",
			XmlError::NoLink => "The link for the channel is not present in the xml passed",
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
	//We check the namespace to make sure we aren't getting content inteded for another
	//kind of app, like atom readers
	parent
		.children()
		.find(|c| c.tag_name().name() == name && c.tag_name().namespace().is_none())
}

fn get_text_from_child(parent: &Node, name: &str) -> Option<String> {
	let borrowed = get_named_child(parent, name)?.text()?;
	Some(String::from(borrowed))
}

fn process_item(item_tag: &Node) -> Option<Item> {
	let title_or_description:String = get_text_from_child(item_tag, "title")
		.or(get_text_from_child(item_tag, "description"))?;

	let link:Option<_> = get_text_from_child(item_tag, "link");

	let pub_date:Option<DateTime<Utc>> = get_text_from_child(item_tag, "pubDate")
		.and_then(|date_s| DateTime::parse_from_rfc2822(&date_s).ok())
		.and_then(|fixed_date| Some(DateTime::<Utc>::from(fixed_date)));

	Some(Item::new(title_or_description, link, pub_date))
}

/// Parses an XML in RSS format into a Channel.
fn parse_rss(root: Node) -> Result<Channel, XmlError> {
	let channel_tag = get_named_child(&root, "channel")
		.ok_or(XmlError::NoChannelTag)?;

	let title = get_text_from_child(&channel_tag, "title")
		.ok_or(XmlError::NoTitle)?;
	let link = get_text_from_child(&channel_tag, "link")
		.ok_or(XmlError::NoLink)?;
	//Some feed generators might not add a description to the channel.
	//While this is technically required by the spec I'd rather be able to parse them.
	//This adds the disadvantage that some errors might go ignored
	let description = get_text_from_child(&channel_tag, "description")
		.unwrap_or(String::new());

	let last_build_date:Option<DateTime<_>> = get_text_from_child(&channel_tag, "lastBuildDate")
		.and_then(|date_s| DateTime::parse_from_rfc2822(&date_s).ok())
		.and_then(|fixed_date| Some(DateTime::<Utc>::from(fixed_date)));

	let items:Vec<Item> = channel_tag .children()
		.filter(|c| c.tag_name().name() == "item")
		.flat_map(|i| process_item(&i))
		.collect();

	Ok(Channel {
		title,
		link,
		description,
		last_build_date,
		items
	})
}


/// Tries to find a direct child of the parent tag with the name _name_
fn get_named_child_atom<'a>(parent: &Node<'a, 'a>, name: &str) -> Option<Node<'a, 'a>> {
	//Maybe we should be normalizing the strings before comparing them
	//We check the namespace to make sure we aren't getting content inteded for another
	//kind of app, like atom readers
	parent
		.children()
		.find(|c| c.tag_name().name() == name && c.tag_name().namespace() == Some("http://www.w3.org/2005/Atom"))
}

fn get_text_from_child_atom(parent: &Node, name: &str) -> Option<String> {
	let borrowed = get_named_child_atom(parent, name)?.text()?;
	Some(String::from(borrowed))
}

/// Parses a single _entry_ block in an atom feed
fn process_atom_entry(entry: &Node) -> Option<Item> {
	// Atom requires entries to have a title, no need to search for a description
	// if one is not present
	let title = get_text_from_child_atom(entry, "title")?;

	let link = get_named_child_atom(entry, "link")
		.and_then(|link_tag| link_tag.attribute("href"))
		.and_then(|href| Some(href.to_string()) );

	// This is less strict than the atom spec, since updated is necessary.
	let pub_date = get_text_from_child_atom(entry, "updated")
		.and_then(|date_s| DateTime::parse_from_rfc3339(&date_s).ok())
		.and_then(|fixed_date| Some(DateTime::<Utc>::from(fixed_date)));

	Some(Item::new(title, link, pub_date))
}

/// Parses an XML in Atom format into a Channel.
fn parse_atom(root: Node) -> Result<Channel, XmlError> {
	let title = get_text_from_child_atom(&root, "title")
		.ok_or(XmlError::NoTitle)?;

	//OBS: DECIDED TO REQUIRE LINK, THIS DOES NOT RESPECT ATOM'S SPEC.
	//Also note that links work like _a_ tags in html.
	let link:String = root.children()
		.find(|c|
			c.tag_name().name() == "link" &&
			c.tag_name().namespace() == Some("http://www.w3.org/2005/Atom") &&
			c.attribute("rel") == Some("self"))
		.ok_or(XmlError::NoLink)?
		.attribute("href")
		.ok_or(XmlError::NoLink)?
		.to_string();

	//OBS: required by RSS spec, not by Atom
	let description = String::new();

	//Not absolutely confident that **all** viable strings (xml:xsd) will be
	//correctly parsed by this.
	let last_build_date:Option<DateTime<_>> = get_text_from_child_atom(&root, "updated")
		.and_then(|date_s| DateTime::parse_from_rfc3339(&date_s).ok())
		.and_then(|fixed_date| Some(DateTime::<Utc>::from(fixed_date)));

	let items:Vec<Item> = root.children()
		.filter(|c| c.tag_name().name() == "entry")
		.flat_map(|i| process_atom_entry(&i))
		.collect();

	Ok(Channel {
		title,
		link,
		description,
		last_build_date,
		items
	})
}

/// Turns an xml string into a Channel struct.
/// Works for both RSS & Atom, though most of Atom's features are ignored.
/// If the xml is malformed/unparsable, an error is returned.
pub fn xml_to_rss(xml_source: &str) -> Result<Channel, XmlError> {
	let xml_tree = roxmltree::Document::parse(xml_source)
		.map_err(|_| XmlError::ParserFailed )?;
	let root = xml_tree.root_element();

	match root.tag_name().namespace() {
		Some("http://www.w3.org/2005/Atom") => parse_atom(root),
		None if root.tag_name().name() == "rss" => parse_rss(root),
		_ => Err(XmlError::UnknownFormat)
	}
}
