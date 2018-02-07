use json::{self, Value};
use errors::ParseError;
use data::{Comment, Listing, Thing};
use failure::Error;
use App;

/// A struct that represents a submission to reddit
#[derive(Debug)]
pub struct Post {
	/// Id of the post
	pub id: String,
	/// Title of the post
	pub title: String,
	/// Author of the post
	pub author: String,
	/// Subreddit the post was made in
	pub subreddit: String,
	/// Number of upvotes the post has recieved
	pub ups: i64,
	/// Number of downvotes the post has recieved
	pub downs: i64,
	/// Total score of the post (ups - downs)
	pub score: i64,
	/// Number of comments on the post
	pub num_comments: i64,
	/// Url of the post
	pub url: String,
	/// Whether the post is stickied
	pub stickied: bool,
	/// Amount of times this post has been gilded
	pub gilded: i64,
	/// The comments on this post
	pub comments: Listing<Comment>,
}

impl Thing for Post {
	fn from_value(val: &Value, app: &App) -> Result<Post, Error> {
		let post = &val["data"]["children"][0]["data"];

		macro_rules! out {
			($val:ident) => {
				return Err(Error::from(ParseError { thing_type: "Post".to_string(), json: json::to_string_pretty($val).unwrap() }));
			};
		}

		let id = match post["id"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let title = match post["title"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let author = match post["author"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let subreddit = match post["subreddit"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let ups = match post["ups"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let downs = match post["downs"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let score = match post["score"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let num_comments = match post["num_comments"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let url = match post["url"].as_str() {
			Some(t) => t.to_string(),
			None => out!(val),
		};
		let stickied = match post["stickied"].as_bool() {
			Some(t) => t,
			None => out!(val),
		};
		let gilded = match post["gilded"].as_i64() {
			Some(t) => t,
			None => out!(val),
		};
		let comments = app.get_comment_tree(&id)?;

		Ok(Post {
			id,
			title,
			author,
			subreddit,
			ups,
			downs,
			score,
			num_comments,
			url,
			stickied,
			gilded,
			comments,
		})
	}
}
