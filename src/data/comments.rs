use json;
use json::Value;

use errors::{Error, ErrorKind, Result, ResultExt};
use data::listing::Listing;

#[derive(Clone)]
pub enum Comment {
	Loaded(Box<CommentData>),
	NotLoaded(String)
}

impl Comment {
	pub fn from_value(val: &Value) -> Result<Comment> {
		let raw = val.clone();
		let val = &val["data"];
		let edited = match val["edited"] {
			Value::Bool(_) => None,
			Value::Number(ref num) => num.as_f64(),
			//&Value::Null => None,
			_ => { panic!("Unexpected value for \"edited\": {}", val["edited"]); }
		};
		let id: String = match val["id"].as_str() {
			Some(t) => t.to_string(),
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let author: String = match val["author"].as_str() {
			Some(t) => t.to_string(),
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let ups: i64 = match val["ups"].as_i64() {
			Some(t) => t,
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let downs: i64 = match val["downs"].as_i64() {
			Some(t) => t,
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let score: i64 = match val["score"].as_i64() {
			Some(t) => t,
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let body: String = match val["body"].as_str() {
			Some(t) => t.to_string(),
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let is_submitter: bool = match val["is_submitter"].as_bool() {
			Some(t) => t,
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let stickied: bool = match val["stickied"].as_bool() {
			Some(t) => t,
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let subreddit: String = match val["subreddit"].as_str() {
			Some(t) => t.to_string(),
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let score_hidden: bool = match val["score_hidden"].as_bool() {
			Some(t) => t,
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let name: String = match val["name"].as_str() {
			Some(t) => t.to_string(),
			None => return Err(ErrorKind::InvalidJson(json::to_string(val).unwrap()).into()),
		};
		let replies: Listing<Comment> = match val["replies"] {
			Value::String(_) => Listing::empty(),
			Value::Object(_) => {
				Listing::from_value(&val["replies"]).unwrap()
			},
			_ => { panic!("Unexpected value for \"replies\": {}", val["replies"])}
		};
		
		Ok(Comment::Loaded(Box::new(CommentData {
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
		})))
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