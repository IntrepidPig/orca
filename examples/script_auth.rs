//! This example shows authorizing as a script to retrieve info about the user authorized.
//!
//! This example requires registering the app as a script at [Reddit](https://www.reddit.com/prefs/apps)

extern crate orca;

use orca::App;

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
	println!("Please enter the requested information");
	let username = input("Username: ");
	let password = input("Password: ");
	let id = input("Client id: ");
	let secret = input("Client secret: ");
	
	let mut reddit = App::new("orca_script_example", "1.0", "/u/IntrepidPig").unwrap();
	reddit.authorize_script(&id, &secret, &username, &password).unwrap();

	let user = reddit.get_self().unwrap();
	println!("Got data: {}", user);
}
