//! This example lets you PM someone from the command line. It also requires setting up your own script
//! app at [Reddit](https://www.reddit.com/prefs/apps). This one loads the variables from the environment.

extern crate orca;

use orca::{App, OAuthApp};

fn get_client_data() -> (String, String) {
	use std::env;
	let id = env::var("ORCA_CLIENT_ID").expect("ORCA_CLIENT_ID must be set");
	let secret = env::var("ORCA_CLIENT_SECRET").expect("ORCA_CLIENT_SECRET must be set");
	(id, secret)
}

fn input(query: &str) -> String {
	use std::io::Write;
	let stdin = std::io::stdin();
	print!("{}", query);
	std::io::stdout().flush().unwrap();
	let mut input = String::new();
	stdin.read_line(&mut input).unwrap();
	input.trim().to_string()
}

fn main() {
	let (id, secret) = get_client_data();
	println!("Please log in.");
	let username = input("Username: ");
	let password = input("Password: ");

	let mut reddit = App::new("orca_pm_example", "1.0", "/u/IntrepidPig").unwrap();

	let auth = OAuthApp::Script {
		id,
		secret,
		username,
		password,
	};
	reddit.authorize(&auth).unwrap();

	println!("Please enter the details of the message.");
	let user = input("To: ");
	let subject = input("Subject: ");
	let message = input("Message: ");

	reddit.message(&user, &subject, &message).unwrap();
}
