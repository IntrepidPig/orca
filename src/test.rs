use super::*;
use std::time::Duration;
use std::thread;

fn init_reddit() -> App {
	use std::env;
	
	let mut reddit = App::new("OrcaLibTest", "v0.0.2", "/u/IntrepidPig/");
	
	let id = match env::var("REDDIT_APP_ID") {
		Ok(id) => id,
		Err(_) => panic!("REDDIT_APP_ID must be set")
	};
	
	let secret = match env::var("REDDIT_APP_SECRET") {
		Ok(secret) => secret,
		Err(_) => panic!("REDDIT_APP_SECRET must be set")
	};
	let username = match env::var("REDDIT_USERNAME") {
		Ok(username) => username,
		Err(_) => panic!("REDDIT_USERNAME must be set")
	};
	let password = match env::var("REDDIT_PASSWORD") {
		Ok(password) => password,
		Err(_) => panic!("REDDIT_PASSWORD must be set")
	};
	
	reddit.conn.auth = Some(reddit.authorize(username, password, net::auth::OauthApp::Script(id, secret)).unwrap());
	
	reddit
}

#[test]
fn get_posts() {
	init_reddit().get_posts("unixporn".to_string(), Sort::Top(SortTime::All)).unwrap();
}

#[test]
fn post_sort() {
	assert_eq!(Sort::Top(SortTime::All).param(), &[("sort", "top"), ("t", "all")])
}

#[test]
fn test_auth() {
	init_reddit().get_user().unwrap();
}

#[test]
fn comment_stream() {
	let mut reddit = init_reddit();
	let comments = reddit.get_comments("all".to_string());
	
	let mut count = 0;
	
	for comment in comments {
		count += 1;
		match comment {
			Comment::Loaded(data) => {
				println!("Got comment #{} by {}", count, data.author);
			},
			_ => { panic!("This was not supposed to happen") }
		}
		
		if count > 128 {
			break;
		};
	};
}

#[test]
fn comment_tree() {
	let mut reddit = init_reddit();
	let tree = reddit.get_comment_tree("6uvvyy".to_string());
	
	fn print_tree(listing: Listing<Comment>, level: i32) {
		for comment in listing {
			match comment {
				Comment::Loaded(data) => {
					for _ in 0..level {
						print!("\t");
					}
					println!("Comment by {}", data.author);
					print_tree(data.replies, level + 1);
				},
				_ => {},
			}
		}
	};
	
	print_tree(tree, 0);
}

//#[test(submit)]
/*
fn test_post() {
	println!("{}", init_reddit().submit_self("pigasusland".to_string(), "Test Post".to_string(), "The time is dank-o-clock".to_string(), true).unwrap());
}*/