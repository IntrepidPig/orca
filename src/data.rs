use json;
use json::Value;

#[derive(Deserialize)]
pub struct Listing<T> {
	pub children: Vec<T>
}

impl Listing<Comment> {
	pub fn from_value(listing: Value) -> Listing<Comment> {
		let mut children: Vec<Comment> = Vec::new();
		
		for comment in listing["data"]["children"].as_array().unwrap() {
			children.push(json::from_value(comment["data"].clone()).unwrap());
		}
		
		Listing { children }
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