use std::collections::VecDeque;

use json;
use json::Value;

use data::{Comment, Thing};
use App;

use failure::Error;
use errors::ParseError;

/// A listing of Things. Has special implementations, currently just for Comments.
#[derive(Debug, Clone)]
pub struct Listing<T> {
	/// The contents of the Listing
	pub children: VecDeque<T>,
}

impl<T> Listing<T> {
	/// Creates a new empty listing
	pub fn new() -> Listing<T> {
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
	/// Flatten this listing of comments (consumes the listing)
	pub fn traverse(self) -> Vec<Comment> {
		let mut comments = Vec::new();

		for comment in self.children.into_iter() {
			comments.push(comment.clone());
			{
				comments.append(&mut comment.replies.traverse());
			}
		}

		comments
	}

	fn insert_comment_recursive(&mut self, comment: Comment) -> bool {
		// For each comment in this listing
		for c in &mut self.children {
			// Check if it's the parent of the comment to be inserted, and if so, insert the comment into the parent's replies
			if c.id == comment.parent_id[3..comment.parent_id.len()] {
				c.replies.children.push_back(comment.clone());
				return true;
			// If not, try to insert it into the replies of the current comment (recursive)
			} else if c.replies.insert_comment_recursive(comment.clone()) {
				return true;
			}
		}

		// The comment was not in this listing
		false
	}

	/// Inserts a comment into a listing in it's correct place in the tree.
	pub fn insert_comment(&mut self, comment: Comment) {
		if !self.insert_comment_recursive(comment.clone()) {
			self.children.push_back(comment);
		}
	}

	/// Parses the listing from json, fetching more comments as necessary.
	pub fn from_value(listing_data: &Value, post_id: &str, app: &App) -> Result<Listing<Comment>, Error> {
		let mut listing: Listing<Comment> = Listing::new();

		if let Some(array) = listing_data.as_array() {
			for item in array {
				let kind = item["kind"].as_str().unwrap();
				if kind == "t1" {
					listing.children.push_back(
						if let Ok(c) = Comment::from_value(
							item,
							app,
						)
						{
							c
						} else {
							return Err(Error::from(ParseError {
								thing_type: "Listing<Comment>".to_string(),
								json: json::to_string_pretty(listing_data).unwrap(),
							}));
						},
					);
				} else if kind == "more" {
					let more = item["data"]["children"].as_array().unwrap();
					let more_id = item["data"]["id"].as_str().unwrap();
					if !more.is_empty() {
						debug!(
							"Need some children {}",
							json::to_string_pretty(more).unwrap()
						);
						let more = more.iter()
							.map(|i| i.as_str().unwrap())
							.collect::<Vec<&str>>();
						for child in app.more_children(post_id, more_id, &more)? {
							listing.children.push_back(child);
						}
						trace!("Successfully got children");
					}
				}
			}

			Ok(listing)
		} else {
			Err(Error::from(ParseError {
				thing_type: "Listing<Comment>".to_string(),
				json: json::to_string_pretty(listing_data).unwrap(),
			}))
		}
	}
}
