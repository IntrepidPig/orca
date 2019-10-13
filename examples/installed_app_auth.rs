//! This example shows authorizing as an installed app to retrieve info about the user authorized.
//!
//! This example requires registering the app as an installed app at [Reddit](https://www.reddit.com/prefs/apps)

use std::{
	env
};

use orca::{
	Reddit,
	net::{
		auth::{Scopes, InstalledAppError},
	}
};

fn var(name: &str) -> String {
	env::var(name).expect(&format!("{} must be set", name))
}

#[tokio::main]
async fn main() {
	env_logger::init().unwrap();
	
	let id = var("ORCA_EXAMPLE_REDDIT_INSTALLED_APP_ID");
	let redirect = var("ORCA_EXAMPLE_REDDIT_INSTALLED_APP_REDIRECT");
	
	let response_gen: std::sync::Arc<dyn Fn(&Result<(), InstalledAppError>) -> hyper::Response<hyper::Body> + Send + Sync> = std::sync::Arc::new(|result| {
		let msg = match result {
			Ok(()) => "Woot! Successfully authorized",
			Err(InstalledAppError::AlreadyRecieved) => "Already authorized somewhere else",
			Err(InstalledAppError::MismatchedState) => "The states did not match",
			Err(InstalledAppError::NeverRecieved) => "Failed to authorize",
			Err(InstalledAppError::Error {
				msg: _msg,
			}) => {
				"The request was not authorized"
			},
		};
		
		hyper::Response::builder()
			.status(hyper::StatusCode::FORBIDDEN)
			.header(
				hyper::header::CONTENT_TYPE,
				hyper::header::HeaderValue::from_str("text/html").unwrap(),
			)
			.body(hyper::Body::from(format!("<h2>{}</h2>", msg)))
			.unwrap()
	});
	
	let scopes = Scopes::all();

	let reddit = Reddit::new("linux", "orca_installed_app_example", "0.0", "/u/IntrepidPig").unwrap();
	reddit.authorize_installed_app(id, redirect, Some(response_gen), scopes).await.unwrap();

	let user_req = hyper::Request::builder()
		.method(hyper::Method::GET)
		.uri("https://oauth.reddit.com/api/v1/me/.json")
		.body(hyper::Body::empty())
		.unwrap();
	
	let user: json::Value = reddit.json_request(user_req).await.unwrap();
	println!("{}", json::to_string_pretty(&user).unwrap());
}
