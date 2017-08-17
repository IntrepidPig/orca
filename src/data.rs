use json;
use json::Value;
use serde::ser::Error;

#[derive(Deserialize)]
pub struct Listing<T> {
	pub children: Vec<T>
}

impl Listing<Comment> {
	pub fn from_value(listing: Value) -> Result<Listing<Comment>, json::Error> {
		let mut children: Vec<Comment> = Vec::new();
		
		if let Some(array) = listing["data"]["children"].as_array() {
			for comment in array {
				children.push(match json::from_value(comment["data"].clone()) {
					Ok(val) => val,
					Err(e) => return Err(e),
				});
			}
			
			Ok(Listing { children })
		} else {
			Err(json::Error::custom("Couldn't parse as array"))
		}
	}
}

#[derive(Deserialize)]
pub struct Comment {
	pub subreddit_id: String,
	pub edited: bool,
	pub link_id: String,
	pub link_author: String,
	pub saved: bool,
	pub id: String,
	pub author: String,
	pub ups: i32,
	pub downs: i32,
	pub score: i32,
	pub parent_id: String,
	pub body: String,
	pub is_submitter: bool,
	pub stickied: bool,
	pub subreddit: String,
	pub score_hidden: bool,
	pub name: String,
	pub replies: Value
}