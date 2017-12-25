use std::collections::HashMap;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

use json;
use json::Value;
use hyper::{Request, Method};
use url::Url;

use net::Connection;
use data::{Comment, Thread, Listing, Thing};
use App;

pub struct Comments<'a> {
	sub: String,
	cache: VecDeque<Thread>,
	last: Option<String>,
	app: &'a App,
}

impl<'a> Comments<'a> {
	// TODO fix all the unwraps
	pub fn new(app: &'a App, sub: &str) -> Comments<'a> {
		let cache: VecDeque<Thread> = VecDeque::new();
		let last = None;

		Comments {
			sub: sub.to_string(),
			cache,
			last,
			app,
		}
	}

	fn refresh(&mut self, app: &App) {
		let mut params: HashMap<String, String> = HashMap::new();
		if let Some(last) = self.last.clone() {
			params.insert("before".to_string(), last);
			params.insert("limit".to_string(), "500".to_string());
		}

		let mut resp = app.get_comments(&self.sub);

		match resp.by_ref().peekable().peek() {
			Some(thread) => {
				match *thread {
					Thread::Comment(ref comment) => {
						self.last = Some(comment.id.clone());
					},
					Thread::More(ref ids) => {
						self.last = Some(ids[0].clone());
					}
				}
			},
			None => {}
		}
		
		self.cache.append(&mut resp.cache);
	}
}

impl<'a> Iterator for Comments<'a> {
	type Item = Thread;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(val) = self.cache.pop_front() {
			Some(val)
		} else {
			while self.cache.is_empty() {
				self.refresh(self.app);
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
