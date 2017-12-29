use std::collections::VecDeque;

use data::Comment;
use App;

pub struct Comments<'a> {
	sub: String,
	cache: VecDeque<Comment>,
	last: Option<String>,
	app: &'a App,
}

impl<'a> Comments<'a> {
	// TODO fix all the unwraps
	pub fn new(app: &'a App, sub: &str) -> Comments<'a> {
		let cache: VecDeque<Comment> = VecDeque::new();
		let last = None;

		Comments {
			sub: sub.to_string(),
			cache,
			last,
			app,
		}
	}

	fn refresh(&mut self, app: &App) {
		let mut resp = app.get_recent_comments(&self.sub, Some(500), self.last.clone())
				.expect("Could not get recent comments");

		match resp.by_ref().peekable().peek() {
			Some(comment) => {
				self.last = Some(comment.id.clone());
			}
			None => {}
		}

		self.cache.append(&mut resp.children);
	}
}

impl<'a> Iterator for Comments<'a> {
	type Item = Comment;

	fn next(&mut self) -> Option<Self::Item> {
		while self.cache.is_empty() {
			self.refresh(self.app);
		}
		self.cache.pop_front()
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
