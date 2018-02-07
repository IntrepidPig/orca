//! This example is the processing of a stream of every comment submitted to Reddit in real time.

extern crate orca;

use orca::App;

fn main() {
	let mut reddit = App::new("orca_stream_example", "1.0", "/u/IntrepidPig").unwrap();
	
	for comment in reddit.create_comment_stream("all") {
		println!("{}: {}\n", comment.author, comment.body);
	}
}