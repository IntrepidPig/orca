extern crate env_logger;

use std::sync::{Arc, Once, ONCE_INIT};
use std::thread;
use std::time::Duration;

use hyper::{Body, Response};
use log;

use auth::OAuth;
use data::*;
use net::LimitMethod;
use *;

static ONCE: Once = ONCE_INIT;

fn init_logging() {
	ONCE.call_once(|| {
		let mut builder = env_logger::LogBuilder::new();
		builder.filter(Some("orca"), log::LogLevelFilter::Trace);
		builder.target(env_logger::LogTarget::Stdout);
		builder.init();
	});
}

fn source_env() -> Result<(String, String, String, String, String, String), ()> {
	use std::env;
	fn get_env(var: &str) -> String {
		match env::var(var) {
			Ok(item) => item,
			_ => panic!("{} must be set", var),
		}
	};

	let username = get_env("REDDIT_USERNAME");
	let password = get_env("REDDIT_PASSWORD");
	let script_id = get_env("REDDIT_SCRIPT_ID");
	let secret = get_env("REDDIT_SCRIPT_SECRET");
	let installed_id = get_env("REDDIT_INSTALLED_ID");
	let redirect = get_env("REDDIT_INSTALLED_REDIRECT");

	Ok((username, password, script_id, secret, installed_id, redirect))
}

fn init_reddit() -> App {
	init_logging();
	let mut reddit = App::new("OrcaLibTest", "v0.2.0", "/u/IntrepidPig").unwrap();
	let (username, password, script_id, secret, installed_id, redirect) = source_env().unwrap();
	reddit.authorize_script(&script_id, &secret, &username, &password).unwrap();

	reddit
}

#[test(posts)]
fn get_posts() {
	init_reddit().get_posts("unixporn", Sort::Top(SortTime::All)).unwrap();
}

// Conflicts with the force_refresh test
//#[test(installed_auth)]
fn installed_app_auth() {
	init_logging();
	let (username, password, script_id, secret, installed_id, redirect) = source_env().unwrap();
	let mut reddit = App::new("Orca Test Installed App", "v0.3.0", "/u/IntrepidPig").unwrap();
	use net::auth::InstalledAppError;
	let response_gen: Arc<ResponseGenFn> = Arc::new(|res: &Result<String, InstalledAppError>| -> Response<Body> {
		match res {
			Ok(code) => Response::new(Body::from("Congratulations! You have been authorized")),
			Err(e) => Response::new(Body::from(format!("ERROR: {}\n\nSorry for the inconvience", e))),
		}
	});
	let mut scopes = Scopes::all();
	scopes.submit = false;

	reddit.authorize_installed_app(&installed_id, &redirect, response_gen, &scopes).unwrap();
	reddit.get_self().unwrap();
	assert!(reddit.submit_self("test", "You shouldn't be seeing this", "Sorry if you do", false).is_err());
}

#[test(sort)]
fn post_sort() {
	init_logging();
	assert_eq!(Sort::Top(SortTime::All).param(), &[("sort", "top"), ("t", "all")])
}

#[test(auth)]
fn test_auth() {
	init_reddit().get_self().unwrap();
}

#[test(selfuser)]
fn self_info() {
	let reddit = init_reddit();

	let user = reddit.get_self().unwrap();
	info!("Me:\n{}", json::to_string_pretty(&user).unwrap());
}

#[test(otheruser)]
fn other_info() {
	let reddit = init_reddit();

	let otherguy = reddit.get_user("DO_U_EVN_SPAGHETTI").unwrap();
	info!("That one guy:\n{}", json::to_string_pretty(&otherguy).unwrap());
}

#[test(stream)]
fn comment_stream() {
	let reddit = init_reddit();
	let comments = reddit.create_comment_stream("all");

	let mut count = 0;

	for comment in comments {
		count += 1;
		trace!("Got comment #{} by {}", count, comment.author);

		if count > 500 {
			break;
		};
	}
}

#[test(tree)]
fn comment_tree() {
	let reddit = init_reddit();
	let tree = reddit.get_comment_tree("7le01h").unwrap();

	fn print_tree(listing: Listing<Comment>, level: i32) {
		for comment in listing {
			for _ in 0..level {
				print!("\t");
			}
			println!("{} by {} (parent: {})", comment.id, comment.author, comment.parent_id);
			print_tree(comment.replies, level + 1);
		}
	};

	print_tree(tree, 0);
}

//#[test(Stress)]
fn stress_test() {
	let requests = 60;

	let reddit = init_reddit();
	reddit.conn.set_limit(LimitMethod::Steady);

	use std::time::{Duration, Instant};

	let mut times: Vec<Duration> = Vec::new();

	let start = Instant::now();
	for userstuff in 0..requests {
		let t1 = Instant::now();
		reddit.get_self().unwrap();
		times.push(Instant::now() - t1);
	}
	let total = Instant::now() - start;

	info!("Total time for {} requests: {:?}", requests, total);

	let mut sum = Duration::new(0, 0);
	for i in times.iter() {
		sum += i.clone();
	}

	info!("Average wait time: {:?}", sum / requests);
}

#[test(Sticky)]
fn sticky() {
	let reddit = init_reddit();
	let name = "t3_6u65br";

	reddit.set_sticky(true, Some(2), name).unwrap();
	thread::sleep(Duration::new(3, 0));
	let post = reddit.load_post(name).unwrap();
	assert!(post.stickied);

	reddit.set_sticky(false, Some(2), name).unwrap();
	thread::sleep(Duration::new(3, 0));
	let post = reddit.load_post(name).unwrap();
	assert!(!post.stickied);
}

#[test(load_post)]
fn load_post() {
	let reddit = init_reddit();

	let post = reddit.load_post("t3_7am0zo").unwrap();
	info!("Got post: {:?}", post);
}

#[test(message)]
fn message() {
	let reddit = init_reddit();

	reddit.message("intrepidpig", "please don't spam me", "oops").unwrap();
}

#[test(submit)]
fn test_post() {
	println!("{}", init_reddit().submit_self("pigasusland", "Test Post", "The time is dank-o-clock", true).unwrap());
}

#[test(urlencode)]
fn urlencode() {
	println!("{}", init_reddit().submit_self("pigasusland", "Tanks & Banks", "Will it work? Cheese & Rice", true).unwrap());
}

#[test(force_refresh)]
fn force_refresh() {
	init_logging();
	let (username, password, script_id, secret, installed_id, redirect) = source_env().unwrap();
	let mut reddit = App::new("Orca Test Installed App", "v0.4.0", "/u/IntrepidPig").unwrap();
	reddit.authorize_installed_app(&installed_id, &redirect, None, &Scopes::all()).unwrap();

	let auth = reddit.conn.auth.as_ref().unwrap();
	let old_auth = auth.clone();
	thread::sleep(Duration::new(2, 0));
	auth.refresh(&reddit.conn).unwrap();
	reddit.get_self().unwrap();
	let new_auth = auth.clone();

	match (old_auth, new_auth) {
		(
			OAuth::InstalledApp {
				id: old_id,
				redirect: old_redirect,
				token: old_token,
				refresh_token: old_refresh_token,
				expire_instant: old_expire_instant,
			},
			OAuth::InstalledApp {
				id: new_id,
				redirect: new_redirect,
				token: new_token,
				refresh_token: new_refresh_token,
				expire_instant: new_expire_instant,
			},
		) => {
			assert_eq!(old_id, new_id);
			assert_eq!(old_redirect, new_redirect);
			assert_ne!(old_token, new_token);
			assert_eq!(old_refresh_token, new_refresh_token);
			assert_ne!(old_expire_instant, new_expire_instant);
		}
		_ => panic!("Got unmatching authorization types"),
	}
}

// Takes over 2 hours
//#[test(auto_refresh)]
fn auto_refresh() {
	init_logging();
	let (username, password, script_id, secret, installed_id, redirect) = source_env().unwrap();
	let mut reddit = App::new("Orca Test Installed App", "v0.4.0", "/u/IntrepidPig").unwrap();
	reddit.authorize_installed_app(&installed_id, &redirect, None, &Scopes::all()).unwrap();
	reddit.get_self().unwrap();

	thread::sleep(Duration::new(60 * 60 + 60, 0)); // Wait a little over an hour
	let mut first = true;
	reddit.get_self().unwrap_or_else(|_| {
		first = false;
		json::Value::Null
	});
	let mut second = true;
	reddit.get_self().unwrap_or_else(|_| {
		second = false;
		json::Value::Null
	});

	thread::sleep(Duration::new(60 * 60 + 60, 0)); // Wait a little over an hour
	let mut third = true;
	reddit.get_self().unwrap_or_else(|_| {
		third = false;
		json::Value::Null
	});
	let mut fourth = true;
	reddit.get_self().unwrap_or_else(|_| {
		fourth = false;
		json::Value::Null
	});

	fn bs(b: bool) -> &'static str {
		if b {
			"Success"
		} else {
			"Failure"
		}
	}

	println!("Tests:\n1: {}\n2: {}\n3: {}\n4: {}", bs(first), bs(second), bs(third), bs(fourth));
	if !(first && second && third && fourth) {
		panic!("Test failed")
	}
}
