//! Abstract away the http petiton logic

use anyhow::{Context, Result};
use reqwest::{Client};
use tokio::{
	runtime::Runtime
};
use url::Url;

use crate::{
	rss::Channel,
	xml_handler::xml_to_rss
};

// A sequential, blocking, request to an RSS feed
//
// Left this function to keep the main.rs code simple and because
// we only call it when the number of petitions is constant, where
// starting tokio is probably not worth it.
pub fn request_feed(feed_source: &str) -> Result<Channel> {
	let parsed_source = Url::parse(feed_source)?;

	let response = reqwest::blocking::get(parsed_source)
		.context(format!("Network request to feed failed for: {feed_source}"))?
		.text()
		.context("Could not turn feed into a string")?;

	xml_to_rss(&response).context("Could not process xml")
}


// Multiple concurrent requests to different RSS feeds.
pub fn concurrent_requests(urls: &[&str]) -> Vec<Result<Channel>> {
	let rt = Runtime::new().expect("Could not start concurrent runtime");
	let client = Client::new();

	rt.block_on(async {
		let handles:Vec<_> = urls.iter().map(|u| async {
			// Needs to be cloned, if not the future outlives the closure ...
			let u = *u;

			let feed_source = Url::parse(u)
				.context(format!("Could not parse {u}"))?;
			let response = client.get(feed_source)
				.send().await
				.context(format!("Could not parse {u}"))?
				.text().await
				.context(format!("Could not turn feed {u} into string"))?;

			xml_to_rss(&response)
				.context(format!("Could not parse xml for feed {u}"))
		}).collect();

		let mut output:Vec<Result<Channel>> = Vec::new();
		for h in handles {
			output.push( h.await );
		}
		output
	})
}
