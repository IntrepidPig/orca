# orca
A simple reddit API interface for Rust

### Features Implemented:
- Comment streams from entire subreddits
- Comment tree traversing
- Oauth script authorization
- Oauth installed app authorization
- Self post submissions
- User info
- Comment data structure
- Listing data structure
- Comment submissions
- Automatic ratelimiting (steady and burst)
- Failure for error handling

### Features Todo (nonexhaustive):
- All data structures, or maybe pure json. Consistency is the goal.
- More reddit api implementation
- More complete error handling


### Contributing
If you've ever made a pull request on github before, you probably know more about it than me. I would really appreciate any help on this project, so if you have an idea on how to improve it, please feel free to submit an issue or pull request.

### Example: Recursively traversing a comment tree
```rust
fn print_tree(listing: Listing<Comment>, level: i32) {
	for comment in listing {
	    for _ in 0..level {
		    print!("\t");
		}
		println!("Comment by {}", comment.author);
		print_tree(comment.replies, level + 1);
	}
}

print_tree(tree, 0);
```

### Example: Authorizing as OAuth Script type
```rust
let mut app = App::new("appnamehere", "v0.1.0", "/u/usernamehere/");
app.authorize_script(id, secret, username, password);
```

> Generic Notice: This is an unstable project yadda yadda yadda use it if you dare thanks
