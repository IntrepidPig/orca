use std::io::Read;
use std::collections::HashMap;

use http::{Method, Request, RequestBuilder, Url};
use http::header::UserAgent;
use json;
use json::Value;

use errors::{Error, ErrorKind, Result, ResultExt};
use net::Connection;


/// Contains data for each possible oauth type
/// Currently only Script is supported
/// # Variants
/// * `Script(String, String)` where String 1 is the script client id and String 2 is the script client password
#[derive(Debug, Clone)]
pub enum OauthApp {
    /// Not Implemented
    WebApp,
    /// ^
    InstalledApp,
    /// Where args are (app id, app secret) is the script secret
    Script(String, String),
}

/// Contains authorization info for a user and client
/// Build with `Auth::new()`
#[derive(Debug, Clone)]
pub struct Auth {
    /// Oauth app type
    pub app: OauthApp,
    /// Username
    pub username: String,
    /// Password
    pub password: String, // TODO not pub
    /// Bearer token
    pub token: String,
}

/// Errors given from authorization
#[derive(Debug, Clone)]
pub enum AuthError<'a> {
    UrlError(&'a str),
    ConnectionError(&'a str),
    RequestError(&'a str),
    ResponseError(&'a str),
}

impl Auth {
    fn get_token(conn: &Connection, app: &OauthApp, username: &str, password: &str) -> Result<String> {
        // TODO: get rid of unwraps and expects
        use self::OauthApp::*;
        match *app {
            Script(ref id, ref secret) => {
                // Request for the bearer token
                let mut tokenreq = conn.client
                    .post("https://ssl.reddit.com/api/v1/access_token") // httpS is important
                    .chain_err(|| "Failed to send request")?;
                // Insert authorization paramaters to request
                let mut params: HashMap<&str, &str> = HashMap::new();
                params.insert("grant_type", "password");
                params.insert("username", username);
                params.insert("password", password);
                // I have no clue what's going on at this point
                let tokenreq = tokenreq
                    .header(conn.useragent.clone())
                    .basic_auth(id.clone(), Some(secret.clone()))
                    .form(&params)
                    .unwrap();
                let mut tokenresponse = tokenreq.send().chain_err(|| "Bad shit")?;
                if tokenresponse.status().is_success() {
                    let mut response = String::new();
                    tokenresponse.read_to_string(&mut response).unwrap();
                    let responsejson: Value = json::from_str(&response).expect("Got response in unknown format");
                    if let Some(token) = responsejson.get("access_token") {
                        let token = token.as_str().unwrap().to_string();
                        Ok(token)
                    } else {
                        Err(ErrorKind::ResponseError(responsejson.to_string()).into())
                    }
                } else {
                    Err(ErrorKind::BadRequest.into())
                }
            }
            // App types other than script are unsupported right now
            _ => Err(ErrorKind::Unimplemented.into()),
        }
    }

    /// Authorize a user
    /// ## To Use:
    /// Set the auth field of a connection to an option with the success value of this function
    /// # Arguments:
    /// * `conn` - A reference to the main connection object (part of a Reddit struct)
    /// * `app` - Oauth app type. This also contains client ids, client secrets, etc.
    /// * `username` - Username of the user to authenticate as
    /// * `password` - Password of the user to authenticate as
    pub fn new(conn: &Connection, app: OauthApp, username: String, password: String) -> Result<Auth> {
        match Auth::get_token(conn, &app, &username, &password) {
            Ok(token) => Ok(Auth {
                app: app,
                username: username,
                password: password,
                token: token,
            }),
            Err(err) => Err(err),
        }
    }
}
