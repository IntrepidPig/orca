use std::collections::HashMap;

use failure::Error;
use hyper::{Body, Request};
use json::Value;
use url::Url;

use data::{Comment, Comments, Listing, Post, Thing};
use net::{body_from_map, uri_params_from_map};
use {App, Sort};

impl App {
	/// Loads a thing and casts it to the type of anything as long as it implements the Thing trait. Experimental
	/// # Arguments
	/// * `fullame` - fullname of the thing
	pub fn load_post(&self, fullname: &str) -> Result<Post, Error> {
		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("names", fullname);

		let req = Request::get(format!("https://www.reddit.com/by_id/{}/.json", fullname)).body(Body::empty()).unwrap();
		let response = self.conn.run_request(req)?;

		Post::from_value(&response, self)
	}

	/// Get the posts in a subreddit sorted in a specific way
	/// # Arguments
	/// * `sub` - Name of subreddit to query
	/// * `sort` - Sort method of query
	/// # Returns
	/// A result containing a json listing of posts
	pub fn get_posts(&self, sub: &str, sort: Sort) -> Result<Value, Error> {
		let req = Request::get(
			Url::parse_with_params(
				&format!(
					"https://www.reddit.com/r/{}/.\
					 json",
					sub
				),
				sort.param(),
			)?
			.into_string(),
		)
		.body(Body::empty())
		.unwrap();

		self.conn.run_request(req)
	}

	/// Get a iterator of all comments in order of being posted
	/// # Arguments
	/// * `sub` - Name of the subreddit to pull comments from. Can be 'all' to pull from all of reddit
	pub fn create_comment_stream(&self, sub: &str) -> Comments {
		Comments::new(self, sub)
	}

	/// Gets the most recent comments in a subreddit. This function is also usually called internally but
	/// can be called if a one time retrieval of recent comments from a subreddit is necessary
	/// # Arguments
	/// * `sub` - Subreddit to load recent comments from
	/// * `limit` - Optional limit to amount of comments loaded
	/// * `before` - Optional comment to be the starting point for the next comments loaded
	/// # Returns
	/// A listing of comments that should be flat (no replies)
	pub fn get_recent_comments(&self, sub: &str, limit: Option<i32>, before: Option<&str>) -> Result<Listing<Comment>, Error> {
		let limit_str;
		let mut params: HashMap<&str, &str> = HashMap::new();
		if let Some(limit) = limit {
			limit_str = limit.to_string();
			params.insert("limit", &limit_str);
		}
		if let Some(ref before) = before {
			params.insert("before", before);
		}

		let req = Request::get(uri_params_from_map(&format!("https://www.reddit.com/r/{}/comments.json", sub), &params)?).body(Body::empty()).unwrap();

		let resp = self.conn.run_request(req)?;
		let comments = Listing::from_value(&resp["data"]["children"], "", self)?;

		Ok(comments)
	}

	/// Loads the comment tree of a post, returning a listing of the Comment enum, which can be
	/// either Loaded or NotLoaded
	/// # Arguments
	/// * `post` - The name of the post to retrieve the tree from
	/// # Returns
	/// A fully populated listing of commments (no `more` values)
	pub fn get_comment_tree(&self, post: &str) -> Result<Listing<Comment>, Error> {
		// TODO add sorting and shit

		let mut params: HashMap<&str, &str> = HashMap::new();
		params.insert("limit", "2147483648");
		params.insert("depth", "2147483648");
		let req = Request::get(format!("https://www.reddit.com/comments/{}/.json", post)).body(body_from_map(&params)).unwrap();

		let data = self.conn.run_request(req)?;
		let data = data[1]["data"]["children"].clone();

		Listing::from_value(&data, post, self)
	}
}
