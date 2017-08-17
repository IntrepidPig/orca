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

pub struct Reddit {
	conn: net::Connection,
}

impl Reddit {
	pub fn new(appname: &str, appversion: &str, appauthor: &str) -> Reddit {
		Reddit {
			conn: net::Connection::new(appname.to_string(), appversion.to_string(), appauthor.to_string())
		}
	}
	
	/// Return an Auth object for use with API calls that require a user account to work
	pub fn authorize(&self, username: String, password: String, oauth: net::auth::OauthApp) -> Result<Auth, AuthError> {
		Auth::new(&self.conn, oauth, username, password)
	}
	
	/// Get the posts in a subreddit sorted in a specific way
	pub fn get_posts(&self, sub: String, sort: sub::Sort) -> Result<json::Value, ()> {
		let req = Request::new(Method::Get,
		                       Url::parse_with_params(&format!("https://www.reddit.com/r/{}/.json", sub),
		                                              sort.param()).unwrap());
		
		self.conn.run_request(req)
	}
	
	/// Submit a self post
	pub fn submit_self(&self, sub: String, title: String, text: String, sendreplies: bool) -> Result<json::Value, ()> {
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
	
	pub fn get_user(&self) -> Result<json::Value, ()> {
		let req = Request::new(Method::Get, Url::parse("https://oauth.reddit.com/api/v1/me/.json").unwrap());
		
		self.conn.run_auth_request(req)
	}
}