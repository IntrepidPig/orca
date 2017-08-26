use json;
use json::Value;

use errors::*;
use data::listing::Listing;

#[derive(Clone)]
pub enum Comment {
	Loaded(CommentData),
	NotLoaded(String)
}

impl Comment {
	pub fn from_value(val: &Value) -> Result<Comment> {
		let raw = val.clone();
		let val = &val["data"];
		let edited = match &val["edited"] {
			&Value::Bool(_) => None,
			&Value::Number(ref num) => num.as_f64(),
			//&Value::Null => None,
			_ => { panic!("Unexpected value for \"edited\": {}", val["edited"]); }
		};
		let id: String = match val["id"].as_str() {
			Some(t) => t.to_string(),
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let author: String = val["author"].as_str().unwrap().to_string();
		let ups: i64 = val["ups"].as_i64().unwrap();
		let downs: i64 = val["downs"].as_i64().unwrap();
		let score: i64 = val["score"].as_i64().unwrap();
		let body: String = val["body"].as_str().unwrap().to_string();
		let is_submitter: bool = val["is_submitter"].as_bool().unwrap();
		let stickied: bool = val["stickied"].as_bool().unwrap();
		let subreddit: String = val["subreddit"].as_str().unwrap().to_string();
		let score_hidden: bool = val["score_hidden"].as_bool().unwrap();
		let name: String = val["name"].as_str().unwrap().to_string();
		let replies: Listing<Comment> = match val["replies"] {
			Value::String(_) => Listing::empty(),
			Value::Object(_) => {
				Listing::from_value(&val["replies"]).unwrap()
			},
			_ => { panic!("Unexpected value for \"replies\": {}", val["replies"])}
		};
		
		Ok(Comment::Loaded(CommentData {
			edited,
			id,
			author,
			ups,
			downs,
			score,
			body,
			is_submitter,
			stickied,
			subreddit,
			score_hidden,
			name,
			replies,
			raw
		}))
	}
}

#[derive(Clone)]
pub struct CommentData {
	pub edited: Option<f64>,
	pub id: String,
	pub author: String,
	pub ups: i64,
	pub downs: i64,
	pub score: i64,
	pub body: String,
	pub is_submitter: bool,
	pub stickied: bool,
	pub subreddit: String,
	pub score_hidden: bool,
	pub name: String,
	pub replies: Listing<Comment>,
	pub raw: Value,
}