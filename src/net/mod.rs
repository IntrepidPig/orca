/// Contains all functionality for OAuth and logins
pub mod auth;

use std::io::Read;
use std::time::{Duration, Instant};
use std::thread;
use std::cell::Cell;

use json;
use json::Value;
use http::{Client, Method, Request, Url};
use http::header::{Authorization, Bearer, UserAgent};

use errors::{Error, ErrorKind, Result, ResultExt};
use self::auth::{Auth, OauthApp};

#[derive(Copy, Clone)]
pub enum LimitMethod {
    Steady,
    Burst,
}

/// A connection holder to reddit. Holds authorization info if provided
pub struct Connection {
    /// Authorization info (optional, but required for sending authorized requests)
    pub auth: Option<auth::Auth>,
    /// User agent for the client
    pub useragent: UserAgent,
    /// HTTP client
    pub client: Client,
    /// How to ratelimit (burst or steady)
    pub limit: Cell<LimitMethod>,
    /// Requests sent in the past minute
    reqspm: Cell<i32>,
    /// Start of the current minute
    lastmin: Cell<Instant>,
    /// Time of last request sent
    lastreq: Cell<Instant>,
}

impl Connection {
    pub fn new(appname: String, appversion: String, appauthor: String) -> Result<Connection> {
        let useragent = UserAgent::new(format!(
            "orca:{}:{} (by {})",
            appname,
            appversion,
            appauthor
        ));
        Ok(Connection {
            auth: None,
            useragent: useragent,
            client: Client::new().unwrap(),
            limit: Cell::new(LimitMethod::Steady),
            reqspm: Cell::new(0),
            lastmin: Cell::new(Instant::now()),
            lastreq: Cell::new(Instant::now()),
        })
    }

    /// Send a request to reddit
    pub fn run_request(&self, req: Request) -> Result<Value> {
        // Reset counter if last minute start is more than a minute away
        if self.lastmin.get().elapsed() > Duration::new(60, 0) {
            self.reqspm.set(0);
            self.lastmin.set(Instant::now());
            println!("Reached new minute");
        }

        // Check if we have reached request limit in this minute, sleep until next minute if necessary
        if self.reqspm.get() >= 30 {
            let targetinstant = self.lastmin.get() + Duration::new(60, 0);
            // Sleep until next minute
            thread::sleep(targetinstant - Instant::now());
        }

        // Ratelimit based on method chosen type
        match self.limit.get() {
            LimitMethod::Steady => {
                // Check if time since last request has been less than 2 seconds, if so then wait necessary time
                if self.lastreq.get().elapsed() < Duration::new(2, 0) {
                    let now = Instant::now();
                    let targetinstant = self.lastreq.get() + Duration::new(2, 0);
                    // Sleep until instant 2 seconds after last request
                    thread::sleep(targetinstant - now);
                }
            }
            LimitMethod::Burst => {
                // Check if reached request per minute limit
                if self.reqspm.get() >= 30 {
                    let targetinstant = self.lastmin.get() + Duration::new(60, 0);
                    // Sleep until next minute
                    thread::sleep(targetinstant - Instant::now());
                }
            }
        };

		// Execute the request!
        let mut response = self.client
            .execute(req)
            .chain_err(|| "Failed to send request")?;
        let mut out = String::new();
        response.read_to_string(&mut out).chain_err(|| "Nice")?;
		
		// Set the last request time to now
        let tmp = Instant::now();
        self.lastreq.set(tmp);

        Ok(json::from_str(&out).chain_err(|| "Couldn't parse json")?)
    }

    /// Send a request to reddit with authorization headers
    pub fn run_auth_request(&self, mut req: Request) -> Result<Value> {
        if let Some(ref auth) = self.auth.clone() {
            req.headers_mut().set(Authorization(Bearer {
                token: auth.token.clone(),
            }));

            self.run_request(req)
        } else {
            Err(ErrorKind::Unauthorized.into())
        }
    }
}
