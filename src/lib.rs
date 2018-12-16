#![deny(missing_docs)]

//! # orca
//! orca is a library to make using the Reddit API from Rust easy
//!
//! ## Features
//! orca has not yet implemented near all of the functionality available in the Reddit API, but
//! enough has been implemented to make simple flexible scripts or apps. Some main functionality
//! includes:
//!
//! * submitting self posts
//! * automatic ratelimiting
//! * commenting and replying
//! * comment streams from subreddits
//! * private messages
//! * authorization as script or installed oauth app type
//! * more stuff
//!
//! ## Structure
//! All of the functionality necessary is available in the implementation of
//! the `App` struct. Data structures are defined in `orca::data`. Networking code is present in
//! the net module, which also contains OAuth authorization functionality.
//!
//! ## Usage
//! To simply create a reddit app instance, do
//!
//! ```rust
//! # use orca::App;
//! # let (name, version, author) = ("a", "b", "c");
//! let mut reddit = App::new(name, version, author).unwrap();
//! ```
//!
//! where `name`, `version`, and `author` are all `&str`s.
//!
//! This instance can do actions that don't require authorization, such as retrieving a stream of
//! comments from a subreddit, but actions such as commenting require authorization, which can be
//! done multiple ways. The most common way for clients to authorize is as scripts, which can be
//! done by just providing a username and password as well as the id and secret of the app that can
//! be registered on the desktop site. It looks like this in code (assuming you already have a
//! mutable reddit instance):
//!
//! ```rust,no_run
//! # use orca::App;
//! # let mut reddit = App::new("a", "b", "c").unwrap();
//! # let (id, secret, username, password) = ("a", "b", "c", "d");
//! reddit.authorize_script(id, secret, username, password).unwrap();
//! ```
//! More info can be found in the documentation for the net module
//!
//! Actually doing something is simple and similar to previous examples. To get info about the
//! currently authorized user, simply call
//!
//! ```rust,no_run
//! # use orca::App;
//! # let mut reddit = App::new("a", "b", "c").unwrap();
//! reddit.get_self();
//! ```
//!
//! which will return a json value until the actual user data structure is implemented.
//!

extern crate chrono;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
extern crate base64;
extern crate open;
extern crate rand;
extern crate serde;
extern crate serde_json as json;
extern crate tokio_core;
extern crate url;

#[cfg(test)]
mod test;

/// Functionality for communication with reddit.com
pub mod net;

/// Reddit data structures
pub mod data;

/// Errors
pub mod errors;

/// Main entry point
pub mod app;

pub use app::App;
pub use data::{Sort, SortTime};
pub use errors::RedditError;
pub use net::auth::{self, InstalledAppError, ResponseGenFn, Scopes};
pub use net::{Connection, LimitMethod};
