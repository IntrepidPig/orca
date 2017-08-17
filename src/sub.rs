use std::collections::HashMap;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use json::Value;
use http::{Request, Method, Url};

use net::Connection;

pub struct Comments<'a> {
	sub: String,
	cache: VecDeque<Value>,
	last: Option<String>,
	conn: &'a Connection,
}

impl<'a> Comments<'a> { // TODO fix all the unwraps
	pub fn new(conn: &'a Connection, sub: String) -> Comments<'a> {
		let cache: VecDeque<Value> = VecDeque::new();
		let last = None;
		
		Comments {
			sub,
			cache,
			last,
			conn
		}
	}
	
	pub fn refresh(&mut self) {
		let mut params: HashMap<String, String> = HashMap::new();
		if let Some(last) = self.last.clone() {
			params.insert("before".to_string(), last);
			params.insert("limit".to_string(), "500".to_string());
		}
		
		let req = Request::new(Method::Get,
		                       Url::parse_with_params(&format!("https://www.reddit.com/r/{}/comments/.json", self.sub), params).unwrap());
		
		let resp = self.conn.run_request(req).unwrap();
		
		self.last = Some(resp["data"]["children"][0]["data"]["name"].as_str().unwrap_or_default().to_string());
		
		let mut new: VecDeque<Value> = VecDeque::from(resp["data"]["children"].as_array().unwrap().to_owned());
		
		self.cache.append(&mut new);
	}
}

impl<'a> Iterator for Comments<'a> {
	type Item = Value;
	
	/// If recieved None, it has already refreshed and recieved no comments. Applications using this
	/// iterator should sleep on recieving None from this function
	fn next(&mut self) -> Option<Self::Item> {
		if let Some(val) = self.cache.pop_front() {
			Some(val)
		} else {
			while self.cache.len() == 0 {
				self.refresh();
				thread::sleep(Duration::from_secs(2));
			}
			self.cache.pop_front()
		}
	}
}

pub enum Sort {
	Hot,
	New,
	Rising,
	Top(SortTime),
	Controversial(SortTime)
}

impl Sort {
	pub fn param<'a>(self) -> Vec<(&'a str, &'a str)> {
		use self::Sort::*;
		match self {
			Hot => {
				vec![("sort", "hot")]
			},
			New => {
				vec![("sort", "new")]
			},
			Rising => {
				vec![("sort", "rising")]
			},
			Top(sort) => {
				vec![("sort", "top"), sort.param()]
			},
			Controversial(sort) => {
				vec![("sort", "controversial"), sort.param()]
			}
		}
	}
}

pub enum SortTime {
	Hour,
	Day,
	Week,
	Month,
	Year,
	All
}

impl SortTime {
	pub fn param<'a>(self) -> (&'a str, &'a str) {
		use self::SortTime::*;
		("t", match self {
			Hour => "hour",
			Day => "day",
			Week => "week",
			Month => "month",
			Year => "year",
			All => "all"
		})
	}
}