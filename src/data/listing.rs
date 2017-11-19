use std::collections::VecDeque;

use json;
use json::Value;

use data::{Comment, CommentData, Thing};

use errors::RedditError;

#[derive(Debug, Clone)]
pub struct Listing<T> {
	pub children: VecDeque<T>,
}

impl<T> Listing<T> {
	pub fn empty() -> Listing<T> {
		Listing { children: VecDeque::new() }
	}
}

impl<T> Iterator for Listing<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.children.pop_front()
	}
}

impl Listing<Comment> {
	pub fn from_value(listing: &Value) -> Result<Listing<Comment>, RedditError> {
		let mut children: VecDeque<Comment> = VecDeque::new();

		if let Some(array) = listing["data"]["children"].as_array() {
			for item in array {
				let kind = item["kind"].as_str().unwrap();
				if kind == "t1" {
					children.push_back(if let Ok(c) = Comment::from_value(item) {
						c
					} else {
						return Err(RedditError::BadResponse { response: listing.to_string() });
					});
				} else if kind == "more" {
					for extra in item["data"]["children"].as_array().unwrap() {
						children.push_back(Comment::NotLoaded(extra.as_str().unwrap().to_string()));
					}
				}
			}

			Ok(Listing { children })
		} else {
			Err(RedditError::BadResponse {
				response: json::to_string(listing).unwrap(),
			})
		}
	}
}
