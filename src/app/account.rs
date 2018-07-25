use failure::Error;
use hyper::{Request, Method};
use url::Url;
use json::Value;

use {App, Sort};

impl App {
	/// Get info of the user currently authorized
	///
	/// Note: requires connection to be authorized
	/// # Returns
	/// A result with the json value of the user data
	pub fn get_self(&self) -> Result<Value, Error> {
		let req = Request::new(
			Method::Get,
			"https://oauth.reddit.com/api/v1/me/.json".parse()?,
		);
		
		self.conn.run_auth_request(req)
	}
}