use std::collections::HashMap;
use std::collections::VecDeque;

use json::Value;

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
		}
		
		let req = self.conn.client.get(&format!("https://www.reddit.com/r/{}/comments/.json", self.sub))
				.unwrap().form(&params).unwrap().build();
		
		let resp = self.conn.run_request(req).unwrap();
		self.last = Some(resp["data"]["after"].as_str().unwrap().to_string());
		
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
			self.refresh();
			if let Some(val) = self.cache.pop_front() {
				Some(val)
			} else {
				None
			}
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