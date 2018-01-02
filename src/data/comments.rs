use json;
use json::Value;

use failure::{Error, err_msg};
use errors::ParseError;
use data::{Listing, Thing};
use App;

/// An enum representing a thread which can either be a comment or a more object that represents
/// a list of comments that have not yet been loaded.
#[derive(Debug, Clone)]
pub enum Thread {
	/// A comment
	Comment(Box<Comment>),
	/// A vector of strings that are the ids of comments that need to be loaded
	More(Vec<String>),
}

/// A struct representing a reddit comment.
/// Does not contain all fields possible in a comment yet.
#[derive(Debug, Clone)]
pub struct Comment {
	/// The amount of seconds since the comment has been edited, if it has been.
	pub edited: Option<f64>,
	/// The id of the comment
	pub id: String,
	/// The id of the comments parent, can be either t1 or t3
	pub parent_id: String,
	/// The link that the comment is present in
	pub link_id: String,
	/// The username of the author of the comment
	pub author: String,
	/// The amount of upvotes the comment has recieved
	pub ups: i64,
	/// The amount of downvotes the comment has recieved
	pub downs: i64,
	/// The score of the comment (ups - downs)
	pub score: i64,
	/// The text of the comment
	pub body: String,
	/// Whether the comment was submitted by the same user that submitted the post
	/// (the author is OP or not)
	pub is_submitter: bool,
	/// Whether the comment is stickied in the thread or not
	pub stickied: bool,
	/// The subreddit the comment was posted in
	pub subreddit: String,
	/// Whether the score of the comment is hidden
	pub score_hidden: bool,
	/// The fullname of the comment (includes the t1_ prefix)
	pub name: String,
	/// A listing of replies to this comment
	pub replies: Listing<Comment>,
}

impl Thing for Comment {
	fn from_value(val: &Value, app: &App) -> Result<Comment, Error> {
		// nice
		macro_rules! out {
			($val:ident) => {
				return Err(Error::from(ParseError { thing_type: "Thread".to_string(), json: json::to_string_pretty($val).unwrap() }));
			};
		}

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
		let parent_id: String = match val["parent_id"].as_str() {
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
		let replies: Listing<Comment> = match val["replies"] {
			Value::String(_) => Listing::new(),
			Value::Object(_) => Listing::from_value(&val["replies"]["data"]["children"], &link_id, app).unwrap(),
			_ => {
				return Err(err_msg(format!(
					"Unexpected value for \"replies\": {}",
					val["replies"]
				)))
			}
		};

		Ok(Comment {
			edited,
			id,
			parent_id,
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
		})
	}
}
