//! # orca
//! orca is a library to make using the Reddit API from Rust easy.
//! 
//! ## Features
//! orca is currently somewhat bare-bones, but it provides a solid foundation
//! to build a Reddit API client on. Some features include:
//! 
//! - Authorization: easily authorize an API client as a script or an installed app
//! - Async: orca allows you to easily execute Reddit API calls concurrently
//!
//! ## Usage
//! To simply create a reddit client instance, do
//!
//! ```rust
//! # use orca::Reddit;
//! # let (platform, id, version, author) = ("a", "b", "c", "d");
//! let mut reddit = Reddit::new(platform, id, version, author)?;
//! ```
//! 
//! Before API calls can be made, the client must be authorized.
//! 
//! ```rust
//! # let (id, secret, username, password) = ("a", "b", "c", "d");
//! reddit.authorize_script(id, secret, username, password).await?;
//! ```
//! 
//! Then to make an API call, create an HTTP request for the specified
//! endpoint, and send the request.
//! 
//! ```rust
//! let user_req = hyper::Request::builder()
//! 	.method(hyper::Method::GET)
//! 	.uri("https://oauth.reddit.com/api/v1/me/.json")
//! 	.body(hyper::Body::empty())?;
//! 
//! let response: serde_json::Value = reddit.json_request(user_req).await?;
//! ```
//!

use std::{
	sync::{RwLock},
};

use hyper::{
	client::{Client, HttpConnector},
	Body,
};
use hyper_tls::{
	HttpsConnector,
};
use snafu::{Snafu};

use crate::{
	net::{
		auth::{OAuth},
	},
};

/// Contains code for handling network communication with reddit (HTTP, ratelimiting, authorization, etc)
pub mod net;
#[cfg(test)]
mod test;

/// A Reddit object. This struct represents the state of a connection with Reddit. It is recommended
/// to only have one Reddit instance per IP address to avoid issues with rate-limiting, but it is not
/// required.
pub struct Reddit {
	auth: RwLock<Option<OAuth>>,
	user_agent: RwLock<String>,
	client: Client<HttpsConnector<HttpConnector>, Body>,
}

impl Reddit {
	/// Create a new Reddit instance
	/// 
	/// ## Parameters
	/// - `platform`: The platform this API client is running on (e.g. linux, android, etc.)
	/// - `app_id`: A unique id to identify this API client
	/// - `app_version`: The current version of this API client
	/// - `app_author`: The author of this API client
	pub fn new(platform: &str, app_id: &str, app_version: &str, app_author: &str) -> Result<Self, hyper_tls::Error> {
		let user_agent = format!("{}:{}:{} (by {})", platform, app_id, app_version, app_author);
		let client = Client::builder()
			.build(HttpsConnector::new()?);
		
		Ok(Self {
			auth: RwLock::new(None),
			user_agent: RwLock::new(user_agent),
			client,
		})
	}
	
	/// Helper function to parse a &str as JSON
	pub fn parse_json<'a, T: serde::Deserialize<'a>>(input: &'a str) -> Result<T, RedditError> {
		json::from_str(input)
			.map_err(|_e| RedditError::BadJson)
	}
}

/// Represents possible errors that can occur while communicating with Reddit
#[derive(Debug, Snafu)]
pub enum RedditError {
	/// Tried to use a User-Agent that isn't valid
	#[snafu(display("User agent was malformed"))]
	BadUserAgent,
	/// An error occurred while sending an HTTP request
	#[snafu(display("An error occurred while sending an HTTP request: {}", source))]
	HttpError {
		/// The underlying HTTP error
		source: hyper::Error,
	},
	/// Tried to parse something as JSON that wasn't valid JSON
	#[snafu(display("Failed to parse text as JSON"))]
	BadJson,
	/// An unknown error occurred
	#[snafu(display("An unknown error occurred"))]
	Unknown,
}