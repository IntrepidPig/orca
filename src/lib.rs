#![allow(unused_imports)]

extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json as json;
extern crate reqwest as http;

use std::fmt;
use std::fmt::Display;
use std::collections::HashMap;

use http::{Request, RequestBuilder, Url, Method};

#[cfg(test)]
mod test;

/// Functionality for communication with reddit.com
pub mod net;

/// Subreddit functionality
pub mod sub;

use net::auth::{Auth, AuthError, OauthApp};

/// A reddit object
/// ## Usage:
/// To create a new instance, use Reddit::new()
pub struct Reddit {
	conn: net::Connection,
}

impl Reddit {
	/// Create a new reddit instance
	/// # Arguments
	/// * `appname` - Unique app name
	/// * `appversion` - App version
	/// * `appauthor` - Auther of the app
	/// # Returns
	/// A new reddit object
	pub fn new(appname: &str, appversion: &str, appauthor: &str) -> Reddit {
		Reddit {
			conn: net::Connection::new(appname.to_string(), appversion.to_string(), appauthor.to_string())
		}
	}
	
	/// Return an Auth object for use with API calls that require a user account to work
	/// # Arguments
	/// * `username` - Username of the user to be authorized as
	/// * `password` - Password of the user to be authorized as
	/// * `oauth` - Oauth app type
	/// # Returns
	/// A result containing either an Auth object or a certain error
	/// To use place it in the auth field of a connection struct
	pub fn authorize(&self, username: String, password: String, oauth: net::auth::OauthApp) -> Result<Auth, AuthError> {
		Auth::new(&self.conn, oauth, username, password)
	}
	
	/// Get the posts in a subreddit sorted in a specific way
	/// # Arguments
	/// * `sub` - Name of subreddit to query
	/// * `sort` - Sort method of query
	/// # Returns
	/// A result containing a json listing of posts
	pub fn get_posts(&mut self, sub: String, sort: sub::Sort) -> Result<json::Value, ()> {
		let req = Request::new(Method::Get,
		                       Url::parse_with_params(&format!("https://www.reddit.com/r/{}/.json", sub),
		                                              sort.param()).unwrap());
		
		self.conn.run_request(req)
	}
	
	/// Submit a self post
	/// # Arguments
	/// * `sub` - Name of the subreddit to submit a post to
	/// * `title` - Title of the post
	/// * `text` - Body of the post
	/// # Returns
	/// A result with reddit's json response to the submission
	pub fn submit_self(&mut self, sub: String, title: String, text: String, sendreplies: bool) -> Result<json::Value, ()> {
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("sr", &sub);
		params.insert("kind", "self");
		params.insert("title", &title);
		params.insert("text", &text);
		params.insert("sendreplies", if sendreplies { "true" } else { "false" });
		
		let req = self.conn.client.post(Url::parse("https://oauth.reddit.com/api/submit/.json").unwrap()).unwrap()
				.form(&params).unwrap().build();
		
		self.conn.run_auth_request(req)
	}
	
	/// Get info of the user currently authorized
	///
	/// Note: requires connection to be authorized
	/// # Returns
	/// A result with the json value of the user data
	pub fn get_user(&mut self) -> Result<json::Value, ()> {
		let req = Request::new(Method::Get, Url::parse("https://oauth.reddit.com/api/v1/me/.json").unwrap());
		
		self.conn.run_auth_request(req)
	}
	
	/// Get a iterator of all comments in order of being posted
	/// # Arguments
	/// * `sub` - Name of the subreddit to pull comments from. Can be 'all' to pull from all of reddit
	pub fn get_comments(&mut self, sub: String) -> sub::Comments {
		sub::Comments::new(&mut self.conn, sub)
	}
}