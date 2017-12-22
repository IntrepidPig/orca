use std::collections::HashMap;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use json;
use json::Value;
use hyper::{Request, Method};
use url::Url;

use net::Connection;
use data::{Comment, CommentData, Listing};

pub struct Comments<'a> {
	sub: String,
	cache: VecDeque<Comment>,
	last: Option<String>,
	conn: &'a Connection,
}

impl<'a> Comments<'a> {
	// TODO fix all the unwraps
	pub fn new(conn: &'a Connection, sub: &str) -> Comments<'a> {
		let cache: VecDeque<Comment> = VecDeque::new();
		let last = None;

		Comments {
			sub: sub.to_string(),
			cache: cache,
			last: last,
			conn: conn,
		}
	}

	fn refresh(&mut self) {
		let mut params: HashMap<String, String> = HashMap::new();
		if let Some(last) = self.last.clone() {
			params.insert("before".to_string(), last);
			params.insert("limit".to_string(), "500".to_string());
		}

		let req = Request::new(
			Method::Get,
			Url::parse_with_params(
				&format!(
					"https://www.reddit.\
                     com/r/{}/comments/.json",
					self.sub
				),
				params,
			).unwrap().into_string().parse().unwrap(), // TODO clean
		);

		let resp = self.conn.run_request(req).unwrap();

		self.last = Some(
			resp["data"]["children"][0]["data"]["name"]
				.as_str()
				.unwrap_or_default()
				.to_string(),
		);

		let mut new: Listing<Comment> = Listing::from_value(&resp).unwrap();

		self.cache.append(&mut new.children);
	}
}

impl<'a> Iterator for Comments<'a> {
	type Item = Comment;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(val) = self.cache.pop_front() {
			Some(val)
		} else {
			while self.cache.is_empty() {
				self.refresh();
			}
			self.cache.pop_front()
		}
	}
}

/// Sort type of a subreddit
pub enum Sort {
	Hot,
	New,
	Rising,
	Top(SortTime),
	Controversial(SortTime),
}

impl Sort {
	/// Convert to url parameters
	pub fn param<'a>(self) -> Vec<(&'a str, &'a str)> {
		use self::Sort::*;
		match self {
			Hot => vec![("sort", "hot")],
			New => vec![("sort", "new")],
			Rising => vec![("sort", "rising")],
			Top(sort) => vec![("sort", "top"), sort.param()],
			Controversial(sort) => vec![("sort", "controversial"), sort.param()],
		}
	}
}

/// Time parameter of a subreddit sort
pub enum SortTime {
	Hour,
	Day,
	Week,
	Month,
	Year,
	All,
}

impl SortTime {
	pub fn param<'a>(self) -> (&'a str, &'a str) {
		use self::SortTime::*;
		(
			"t",
			match self {
				Hour => "hour",
				Day => "day",
				Week => "week",
				Month => "month",
				Year => "year",
				All => "all",
			},
		)
	}
}
