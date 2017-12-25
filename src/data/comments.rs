use json;
use json::Value;

use failure::{err_msg, Error};
use errors::ParseError;
use data::{Post, Listing, Thing};
use App;

#[derive(Debug, Clone)]
pub enum Thread {
	Comment(Box<Comment>),
	More(Vec<String>)
}

#[derive(Debug, Clone)]
pub struct Comment {
	pub edited: Option<f64>,
	pub id: String,
	pub link_id: String,
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
	pub replies: Listing<Thread>,
	pub raw: Value,
}

impl Thing for Thread {
	fn from_value(val: &Value, app: &App) -> Result<Thread, Error> {
		// nice
		macro_rules! out {
			($val:ident) => {
				return Err(Error::from(ParseError { thing_type: "Thread".to_string(), json: $val.clone() }));
			};
		}

		let raw = val.clone();
		let val = &val["data"];
		let edited = match val["edited"] {
			Value::Bool(_) => None,
			Value::Number(ref num) => num.as_f64(),
			Value::Null => None,
			_ => panic!("Unexpected value for \"edited\": {}", val["edited"]),
		};
		let id: String = match val["id"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let link_id: String = match val["link_id"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let author: String = match val["author"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let ups: i64 = match val["ups"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let downs: i64 = match val["downs"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let score: i64 = match val["score"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let body: String = match val["body"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let is_submitter: bool = match val["is_submitter"].as_bool() {
			Some(t) => t,
			None => out!(val),
		};
		let stickied: bool = match val["stickied"].as_bool() {
			Some(t) => t,
			None => out!(val),
		};
		let subreddit: String = match val["subreddit"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let score_hidden: bool = match val["score_hidden"].as_bool() {
			Some(t) => t,
			None => out!(val),
		};
		let name: String = match val["name"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let replies: Listing<Thread> = match val["replies"] {
			Value::String(_) => Listing::empty(),
			Value::Object(_) => Listing::from_value(&val["replies"]["data"]["children"], &link_id, app).unwrap(),
			_ => return Err(err_msg(format!("Unexpected value for \"replies\": {}", val["replies"]))),
		};

		Ok(Thread::Comment(Box::new(Comment {
			edited,
			id,
			link_id,
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
			raw,
		})))
	}

	fn get_json(&self) -> &Value {
		match self {
			&Thread::Comment(ref data) => &data.raw,
			&Thread::More(ref _ids) => {
				// TODO fix
				panic!("Shit");
			}
		}
	}
}
