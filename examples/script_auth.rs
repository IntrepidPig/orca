//! This example shows authorizing as a script to retrieve info about the user authorized.
//!
//! This example requires registering the app as a script at [Reddit](https://www.reddit.com/prefs/apps)

use std::{
	env,
};

use orca::Reddit;

fn var(name: &str) -> String {
	env::var(name).expect(&format!("{} must be set", name))
}

#[tokio::main]
async fn main() {
	env_logger::init().unwrap();
	
	let username = var("ORCA_EXAMPLE_REDDIT_USERNAME");
	let password = var("ORCA_EXAMPLE_REDDIT_PASSWORD");
	let id = var("ORCA_EXAMPLE_REDDIT_SCRIPT_ID");
	let secret = var("ORCA_EXAMPLE_REDDIT_SCRIPT_SECRET");

	let reddit = Reddit::new("linux", "orca_script_example", "0.0", "/u/IntrepidPig").unwrap();
	reddit.authorize_script(id, secret,	username, password).await.unwrap();

	let user_req = hyper::Request::builder()
		.method(hyper::Method::GET)
		.uri("https://oauth.reddit.com/api/v1/me/.json")
		.body(hyper::Body::empty())
		.unwrap();
	
	let user: json::Value = reddit.json_request(user_req).await.unwrap();
	println!("{}", json::to_string_pretty(&user).unwrap());
}
