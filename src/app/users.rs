use failure::Error;
use hyper::{Request, Method};
use url::Url;
use json::Value;

use {App, Sort};

impl App {
	/// Gets information about a user that is not currently authorized
	/// # Arguments
	/// * `name` - username of the user to query
	/// # Returns
	/// A json value containing the user info
	pub fn get_user(&self, name: &str) -> Result<Value, Error> {
		let req = Request::new(
			Method::Get,
			format!("https://www.reddit.com/user/{}/about/.json", name).parse()?,
		);
		
		self.conn.run_request(req)
	}
}