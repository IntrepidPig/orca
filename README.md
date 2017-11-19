# orca
A simple reddit API interface for Rust

### Features Implemented:
- Comment streams from entire subreddits
- Comment tree traversing
- Oauth script authorization
- Self post submissions
- User info
- Comment data structure
- Listing data structure
- Comment submissions
- Automatic ratelimiting (steady and burst)
- Failure for error handling

### Features Todo (nonexhaustive):
- All data structures, or maybe pure json. Consistency is the goal.
- Oauth installed app type
- Better documentation
- More reddit api implementation


### Contributing
If you've ever made a pull request on github before, you probably know more about it than me. I would really appreciate any help on this project, so if you have an idea on how to improve it, please feel free to submit an issue or pull request.

### Example: Recursively traversing a comment tree
```rust
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
}
print_tree(tree, 0);
```


### Example: Authorizing as Ouath Script type
```rust
let mut app = App::new("appnamehere", "v0.0.2", "/u/usernamehere/");
reddit.conn.auth = Some(reddit.authorize(username, password, OauthApp::Script(app_id, app_secret)).unwrap());
```

> Generic Notice: This is an unstable project yadda yadda yadda use it if you dare thanks
