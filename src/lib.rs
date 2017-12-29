#![feature(nll)]

extern crate chrono;
#[macro_use]
extern crate failure_derive;
extern crate failure;
extern crate serde;
extern crate serde_json as json;
extern crate open;
extern crate tiny_http;
extern crate url;
extern crate rand;
extern crate hyper;
extern crate tokio_core;
extern crate futures;
extern crate hyper_tls;
#[macro_use]
extern crate log;

use std::collections::{HashMap, VecDeque};

use json::Value;
use hyper::{Request, Method};

#[cfg(test)]
mod test;

/// Functionality for communication with reddit.com
pub mod net;

/// Reddit data structures
pub mod data;

/// Errors
pub mod errors;
use errors::RedditError;

use failure::Error;
use url::Url;

use net::{Connection, body_from_map, uri_params_from_map};
use net::auth::OAuth;
use data::{Comment, Comments, Listing, Sort, Thing};

/// A reddit object
/// ## Usage:
/// To create a new instance, use `Reddit::new()`
pub struct App {
	pub conn: net::Connection,
}

impl App {
	/// Create a new reddit instance
	/// # Arguments
	/// * `appname` - Unique app name
	/// * `appversion` - App version
	/// * `appauthor` - Auther of the app
	/// # Returns
	/// A new reddit object
	pub fn new(appname: &str, appversion: &str, appauthor: &str) -> Result<App, Error> {
		Ok(App {
			conn: Connection::new(appname, appversion, appauthor)?,
		})
	}

	/// Return an Auth object for use with API calls that require a user account to work
	/// # Arguments
	/// * `username` - Username of the user to be authorized as
	/// * `password` - Password of the user to be authorized as
	/// * `oauth` - Oauth app type
	/// # Returns
	/// A result containing either an Auth object or a certain error
	/// To use place it in the auth field of a connection struct
	pub fn authorize(&mut self, oauth: &net::auth::OauthApp) -> Result<(), Error> {
		self.conn.auth = Some(OAuth::new(&self.conn, oauth)?);
		Ok(())
	}

	/// Get the posts in a subreddit sorted in a specific way
	/// # Arguments
	/// * `sub` - Name of subreddit to query
	/// * `sort` - Sort method of query
	/// # Returns
	/// A result containing a json listing of posts
	pub fn get_posts(&self, sub: &str, sort: Sort) -> Result<Value, Error> {
		let req = Request::new(
			Method::Get,
			Url::parse_with_params(
				&format!(
					"https://www.reddit.com/r/{}/.\
                     json",
					sub
				),
				sort.param(),
			)?
				.into_string()
				.parse()?, // TODO clean
		);

		self.conn.run_request(req)
	}

	/// Submit a self post
	/// # Arguments
	/// * `sub` - Name of the subreddit to submit a post to
	/// * `title` - Title of the post
	/// * `text` - Body of the post
	/// # Returns
	/// A result with reddit's json response to the submission
	pub fn submit_self(&self, sub: &str, title: &str, text: &str, sendreplies: bool) -> Result<Value, Error> {
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("sr", sub);
		params.insert("kind", "self");
		params.insert("title", title);
		params.insert("text", text);
		params.insert("sendreplies", if sendreplies { "true" } else { "false" });

		let mut req = Request::new(
			Method::Post,
			"https://oauth.reddit.com/api/submit/.json".parse()?,
		);
		req.set_body(body_from_map(&params));

		self.conn.run_auth_request(req)
	}

	/// Get info of the user currently authorized
	///
	/// Note: requires connection to be authorized
	/// # Returns
	/// A result with the json value of the user data
	pub fn get_self(&self) -> Result<Value, Error> {
		let req = Request::new(
			Method::Get,
			"https://oauth.reddit.com/api/v1/me/.json".parse()?,
		);

		self.conn.run_auth_request(req)
	}

	pub fn get_user(&self, name: &str) -> Result<Value, Error> {
		let req = Request::new(
			Method::Get,
			format!("https://www.reddit.com/user/{}/about/.json", name)
				.parse()?,
		);

		self.conn.run_request(req)
	}

	/// Get a iterator of all comments in order of being posted
	/// # Arguments
	/// * `sub` - Name of the subreddit to pull comments from. Can be 'all' to pull from all of reddit
	pub fn create_comment_stream(&self, sub: &str) -> Comments {
		Comments::new(self, sub)
	}
	
	pub fn get_recent_comments(&self, sub: &str, limit: Option<i32>, before: Option<String>) -> Result<Listing<Comment>, Error> {
		let limit_str;
		let before_str;
		let mut params: HashMap<&str, &str> = HashMap::new();
		if let Some(limit) = limit {
			limit_str = limit.to_string();
			params.insert("limit", &limit_str);
		}
		if let Some(before) = before {
			before_str = before;
			params.insert("before", &before_str);
		}
		
		let req = Request::new(Method::Get, uri_params_from_map(&format!("https://www.reddit.com/r/{}/comments.json", sub), &params)?);
		
		let resp = self.conn.run_request(req)?;
		let comments = Listing::from_value(&resp["data"]["children"], "", self)?;
		
		Ok(comments)
	}

	/// Loads the comment tree of a post, returning a listing of the Comment enum, which can be
	/// either Loaded or NotLoaded
	/// # Arguments
	/// * `post` - The name of the post to retrieve the tree from
	pub fn get_comment_tree(&self, post: &str) -> Result<Listing<Comment>, Error> {
		// TODO add sorting and shit
		let mut req = Request::new(
			Method::Get,
			format!("https://www.reddit.com/comments/{}/.json", post)
				.parse()?,
		);

		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("limit", "2147483648");
		params.insert("depth", "2147483648");
		req.set_body(body_from_map(&params));


		let data = self.conn.run_request(req)?;
		let data = data[1]["data"]["children"].clone();

		Listing::from_value(&data, post, &self)
	}

	/// Load more comments
	pub fn more_children(&self, link_id: &str, morechildren_id: &str, comments: &[&str]) -> Result<Listing<Comment>, Error> {
		let mut string = String::from("t3_");
		let link_id = if !link_id.starts_with("t3_") {
			string.push_str(link_id);
			&string
		} else {
			link_id
		};
		
		let limit = 5;
		// Break requests into chunks of `limit`
		let mut chunks: Vec<String> = Vec::new();
		let mut chunk_buf = String::new();
		for (i, id) in comments.iter().enumerate() {
			if i != 0 && i % limit == 0 {
				chunk_buf.pop(); // Removes trailing comma
				chunks.push(chunk_buf);
				chunk_buf = String::new();
			}

			chunk_buf.push_str(&format!("{},", id));
		}
		chunk_buf.pop(); // Removes trailing comma on unfinished chunk
		chunks.push(chunk_buf);

		trace!("Chunks are {:?}", chunks);

		let mut lists = Vec::new();

		for chunk in chunks {
			let mut params: HashMap<&str, &str> = HashMap::new();
			params.insert("children", &chunk);
			params.insert("link_id", link_id);
			params.insert("id", morechildren_id);
			params.insert("api_type", "json");

			trace!("Getting more children {} from {}", chunk, link_id);

			//let mut req = Request::new(Method::Get, Url::parse_with_params("https://www.reddit.com/api/morechildren/.json", params)?.into_string().parse()?);
			let mut req = Request::new(
				Method::Post,
				"https://www.reddit.com/api/morechildren/.json".parse()?,
			);
			req.set_body(body_from_map(&params));
			let data = self.conn.run_request(req)?;

			trace!("Scanning {}", data);
			
			let list: Listing<Comment> = Listing::from_value(&data["json"]["data"]["things"], link_id, self)?;
			lists.push(list);
		}
		
		// Flatten the vec of listings
		let mut final_list = VecDeque::new();
		for list in &mut lists {
			final_list.append(&mut list.children);
		}
		let mut listing: Listing<Comment> = Listing::new();
		
		for comment in final_list {
			listing.insert_comment(comment);
		}
		
		Ok(listing)
	}

	/// Comment on a thing
	/// # Arguments
	/// * `text` - The body of the comment
	/// * `thing` - Fullname of the thing to comment on
	pub fn comment(&self, text: &str, thing: &str) -> Result<(), Error> {
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("text", text);
		params.insert("thing_id", thing);

		let mut req = Request::new(
			Method::Post,
			"https://oauth.reddit.com/api/comment".parse()?,
		);
		req.set_body(body_from_map(&params));

		self.conn.run_auth_request(req)?;
		Ok(())
	}

	/// Sticky a post in a subreddit
	/// # Arguments
	/// * `sticky` - boolean value. True to set post as sticky, false to unset post as sticky
	/// * `slot` - Optional slot number to fill (1 or 2)
	/// * `id` - _fullname_ of the post to sticky
	#[allow(unused_must_use)] // Is allowed because the request errors if the post is already sticky but that's ok
	pub fn set_sticky(&self, sticky: bool, slot: Option<i32>, id: &str) -> Result<(), Error> {
		let numstr;
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("state", if sticky { "1" } else { "0" });

		if let Some(num) = slot {
			if num != 1 && num != 2 {
				return Err(Error::from(RedditError::BadRequest {
					request: format!("Sticky's are limited to slots 1 and 2"),
					response: "not sent".to_string(),
				}));
			}
			numstr = num.to_string();
			params.insert("num", &numstr);
		}

		params.insert("id", id);

		let mut req = Request::new(
			Method::Post,
			"https://oauth.reddit.com/api/set_subreddit_sticky/.json"
				.parse()?,
		);
		req.set_body(body_from_map(&params));
		
		self.conn.run_auth_request(req);
		
		Ok(())
	}

	/// Load a thing
	pub fn load_thing<T>(&self, fullname: &str) -> Result<T, Error>
	where
		T: Thing,
	{
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("names", fullname);

		let req = Request::new(
			Method::Get,
			format!("https://www.reddit.com/by_id/{}/.json", fullname)
				.parse()?,
		);
		let response = self.conn.run_request(req)?;

		T::from_value(&response, self)
	}

	pub fn message(&self, to: &str, subject: &str, body: &str) -> Result<(), Error> {
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("to", to);
		params.insert("subject", subject);
		params.insert("text", body);

		let mut req = Request::new(
			Method::Post,
			"https://oauth.reddit.com/api/compose/.json".parse()?,
		);
		req.set_body(body_from_map(&params));

		match self.conn.run_auth_request(req) {
			Ok(_) => Ok(()),
			Err(e) => Err(e),
		}
	}
}
