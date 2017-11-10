#![allow(unused_imports)]

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate reqwest as http;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json as json;

use std::fmt;
use std::fmt::Display;
use std::collections::HashMap;

use json::Value;
use http::{Method, Request, RequestBuilder, Url};

#[cfg(test)]
mod test;

/// Functionality for communication with reddit.com
pub mod net;

/// Reddit data structures
pub mod data;

/// Errors
pub mod errors;
use errors::{Error, ErrorKind, Result, ResultExt};

use net::Connection;
use net::auth::{Auth, AuthError, OauthApp};
use data::{Comment, CommentData, Comments, Listing, Sort, SortTime};

/// A reddit object
/// ## Usage:
/// To create a new instance, use `Reddit::new()`
pub struct App {
    pub conn: net::Connection,
}

impl App {
    /// Create a new reddit instance
    /// # Arguments
    /// * `appname` - Unique app name
    /// * `appversion` - App version
    /// * `appauthor` - Auther of the app
    /// # Returns
    /// A new reddit object
    pub fn new(appname: &str, appversion: &str, appauthor: &str) -> Result<App> {
        Ok(App {
            conn: Connection::new(
                appname.to_string(),
                appversion.to_string(),
                appauthor.to_string(),
            ).chain_err(|| "Failed to initialize reddit connection")?,
        })
    }

    /// Return an Auth object for use with API calls that require a user account to work
    /// # Arguments
    /// * `username` - Username of the user to be authorized as
    /// * `password` - Password of the user to be authorized as
    /// * `oauth` - Oauth app type
    /// # Returns
    /// A result containing either an Auth object or a certain error
    /// To use place it in the auth field of a connection struct
    pub fn authorize(&self, username: String, password: String, oauth: net::auth::OauthApp) -> Result<Auth> {
        Auth::new(&self.conn, oauth, username, password)
    }

    /// Get the posts in a subreddit sorted in a specific way
    /// # Arguments
    /// * `sub` - Name of subreddit to query
    /// * `sort` - Sort method of query
    /// # Returns
    /// A result containing a json listing of posts
    pub fn get_posts(&self, sub: String, sort: Sort) -> Result<Value> {
        let req = Request::new(
            Method::Get,
            Url::parse_with_params(
                &format!(
                    "https://www.reddit.com/r/{}/.\
                     json",
                    sub
                ),
                sort.param(),
            ).unwrap(),
        );

        self.conn.run_request(req)
    }

    /// Submit a self post
    /// # Arguments
    /// * `sub` - Name of the subreddit to submit a post to
    /// * `title` - Title of the post
    /// * `text` - Body of the post
    /// # Returns
    /// A result with reddit's json response to the submission
    pub fn submit_self(&self, sub: String, title: String, text: String, sendreplies: bool) -> Result<Value> {
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("sr", &sub);
        params.insert("kind", "self");
        params.insert("title", &title);
        params.insert("text", &text);
        params.insert("sendreplies", if sendreplies { "true" } else { "false" });

        let req = self.conn
            .client
            .post(Url::parse("https://oauth.reddit.com/api/submit/.json").unwrap())
            .unwrap()
            .form(&params)
            .unwrap()
            .build();

        self.conn.run_auth_request(req)
    }

    /// Get info of the user currently authorized
    ///
    /// Note: requires connection to be authorized
    /// # Returns
    /// A result with the json value of the user data
    pub fn get_self(&self) -> Result<Value> {
        let req = Request::new(
            Method::Get,
            Url::parse("https://oauth.reddit.com/api/v1/me/.json").unwrap(),
        );

        self.conn.run_auth_request(req)
    }

    pub fn get_user(&self, name: &str) -> Result<Value> {
        let req = Request::new(
            Method::Get,
            Url::parse(&format!("https://www.reddit.com/user/{}/about/.json", name)).unwrap(),
        );

        self.conn.run_request(req)
    }

    /// Get a iterator of all comments in order of being posted
    /// # Arguments
    /// * `sub` - Name of the subreddit to pull comments from. Can be 'all' to pull from all of reddit
    pub fn get_comments(&self, sub: String) -> Comments {
        Comments::new(&self.conn, sub)
    }

    /// Loads the comment tree of a post, returning a listing of the Comment enum, which can be
    /// either Loaded or NotLoaded
    /// # Arguments
    /// * `post` - The name of the post to retrieve the tree from
    pub fn get_comment_tree(&self, post: String) -> Listing<Comment> {
        // TODO add sorting and shit
        let req = self.conn
            .client
            .get(Url::parse(&format!("https://www.reddit.com/comments/{}/.json", post)).unwrap())
            .unwrap()
            .build();

        let data = self.conn.run_request(req).unwrap();
        let data = data[1].clone();

        Listing::from_value(&data).expect("failed to parse listing")
    }

    /// Load more comments
    pub fn more_children(&self, comment: &str) { //-> Listing<Comment> {

    }

    /// Comment on a thing
    /// # Arguments
    /// * `text` - The body of the comment
    /// * `thing` - Fullname of the thing to comment on
    pub fn comment(&self, text: String, thing: String) {
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("text", &text);
        params.insert("thing_id", &thing);

        let req = self.conn
            .client
            .post(Url::parse("https://oauth.reddit.com/api/comment").unwrap())
            .unwrap()
            .form(&params)
            .unwrap()
            .build();

        self.conn.run_auth_request(req).unwrap();
    }

    /// Sticky a post in a subreddit
    /// # Arguments
    /// * `sticky` - boolean value. True to set post as sticky, false to unset post as sticky
    /// * `slot` - Optional slot number to fill (1 or 2)
    /// * `id` - id of the post to sticky
    pub fn set_sticky(&self, sticky: bool, slot: Option<i32>, id: &str) -> Result<()> {
        let numstr;
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert(
            "state",
            if sticky { "true" } else { "false" },
        );

        if let Some(num) = slot {
            if num != 1 && num != 2 {
                return Err(ErrorKind::Other("Slot must be 1 or 2".to_string()).into());
            }
            numstr = num.to_string();
            params.insert("num", &numstr);
        }

        params.insert("id", id);

        let req = self.conn
            .client
            .post(Url::parse("https://oauth.reddit.com/api/set_subreddit_sticky").unwrap())
            .unwrap()
            .form(&params)
            .unwrap()
            .build();

        self.conn.run_auth_request(req)?;

        Ok(())
    }
}
