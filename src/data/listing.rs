use std::collections::VecDeque;

use json;
use json::Value;

use data::{Comment, Thread, Thing};
use App;

use failure::Error;
use errors::ParseError;


#[derive(Debug, Clone)]
pub struct Listing<T> {
	pub children: VecDeque<T>,
	pub raw: Value
}

impl<T> Listing<T> {
	pub fn empty() -> Listing<T> {
		Listing { children: VecDeque::new(), raw: json!({
			"kind": "Listing",
			"data": {
				"modhash": "",
				"whitelist_status": "all_ads",
				"children": [],
				"after": null,
				"before": null
			}
		})}
	}
}

impl<T> Iterator for Listing<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.children.pop_front()
	}
}

impl Listing<Thread> {
	pub fn from_value(listing: &Value, post_id: &str, app: &App) -> Result<Listing<Thread>, Error> {
		let mut children: VecDeque<Thread> = VecDeque::new();
		
		if let Some(array) = listing.as_array() {
			for item in array {
				let kind = item["kind"].as_str().unwrap();
				if kind == "t1" {
					//let post_id = listing[0]["data"]["link_id"].as_str().unwrap();
					children.push_back(if let Ok(c) = Thread::from_value(item, app) {
						c
					} else {
						return Err(Error::from(ParseError { thing_type: "Listing<Thread>".to_string(), json: listing.clone() }));
					});
				} else if kind == "more" {
					let more = item["data"]["children"].as_array().unwrap();
					println!("Need some children {}", json::to_string_pretty(more).unwrap());
					let more = more.iter().map(|i| { i.as_str().unwrap() }).collect::<Vec<&str>>();
					for child in app.more_children(post_id, &more)? {
						println!("Got more children {:?}", child);
						children.push_back(child)
					}
					println!("Successfully got children");
				}
			}

			Ok(Listing { children, raw: listing.clone() })
		} else {
			Err(Error::from(ParseError {
				thing_type: "Listing<Thread>".to_string(),
				json: listing.clone(),
			}))
		}
	}
	
	pub fn from_morecomment_value(listing: &Value, post_id: &str, app: &App) {
	
	}
	
	fn get_json(&self) -> &Value {
		&self.raw
	}
}

fn find_listing(data: &Value) {//-> Listing<Thread> {

}
