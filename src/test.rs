extern crate env_logger;

use super::*;
use std::sync::{Once, ONCE_INIT};
use std::time::Duration;
use std::thread;
use net::LimitMethod;
use data::*;
use log;

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

	Ok((
		username,
		password,
		script_id,
		secret,
		installed_id,
		redirect,
	))
}

fn init_reddit() -> App {
	init_logging();
	let mut reddit = App::new("OrcaLibTest", "v0.2.0", "/u/IntrepidPig").unwrap();
	let (username, password, script_id, secret, installed_id, redirect) = source_env().unwrap();
	reddit
		.authorize(net::auth::OauthApp::Script {
			id: script_id,
			secret,
			username,
			password,
		})
		.unwrap();

	reddit
}

#[test(posts)]
fn get_posts() {
	init_reddit()
		.get_posts("unixporn", Sort::Top(SortTime::All))
		.unwrap();
}

#[test(installed_auth)]
fn installed_app_auth() {
	init_logging();
	let (username, password, script_id, secret, installed_id, redirect) = source_env().unwrap();
	let mut reddit = App::new("Orca Test Installed App", "v0.2.0", "/u/IntrepidPig").unwrap();
	reddit
		.authorize(net::auth::OauthApp::InstalledApp {
			id: installed_id,
			redirect,
			error_response: None,
			success_response: None,
		})
		.unwrap();

	reddit.get_self().unwrap();
}

#[test(sort)]
fn post_sort() {
	init_logging();
	assert_eq!(
		Sort::Top(SortTime::All).param(),
		&[("sort", "top"), ("t", "all")]
	)
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
	info!(
		"That one guy:\n{}",
		json::to_string_pretty(&otherguy).unwrap()
	);
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
			println!(
				"{} by {} (parent: {})",
				comment.id,
				comment.author,
				comment.parent_id
			);
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

	reddit.set_sticky(true, Some(2), "t3_6u65br").unwrap();
	thread::sleep(Duration::new(5, 0));

	thread::sleep(Duration::new(10, 0));
	reddit.set_sticky(false, Some(2), "t3_6u65br").unwrap();
}

#[test(load_post)]
fn load_post() {
	let reddit = init_reddit();

	let post = reddit.load_post("t3_7am0zo").unwrap();
	info!("Got post: {:?}", post);
}

//#[test(message)]
fn message() {
	let reddit = init_reddit();

	reddit
		.message("intrepidpig", "please don't spam me", "oops")
		.unwrap();
}

#[test(submit)]
fn test_post() {
	println!(
		"{}",
		init_reddit()
			.submit_self("pigasusland", "Test Post", "The time is dank-o-clock", true)
			.unwrap()
	);
}
