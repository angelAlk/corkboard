//! RSS client
//!
//! Update feeds and keep track of which articles have been read
//! (without being too intrusive with what hasn't been)

use anyhow::Result;
pub mod rss;
mod xml_handler;
//cli
//databse
//http maybe ?

fn main() -> Result<()>{
	//let body = reqwest::blocking::get("https://astralcodexten.substack.com/feed")?.text()?;
	let body = std::fs::read_to_string("assets/sample1.rss")?;

	let channel = xml_handler::xml_to_rss(&body)?;

	//request rss
	//parse xml
	//register into db
	//display result
	Ok(())
}

//turn your brain on
