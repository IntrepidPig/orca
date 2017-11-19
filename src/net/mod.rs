/// Contains all functionality for OAuth and logins
pub mod auth;
/// Reddit errors
pub mod error;

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
    /// Requests sent in the past ratelimit period
    reqs: Cell<i32>,
    /// Requests remaining
    remaining: Cell<Option<i32>>,
    /// Time when request amount will reset
    reset_time: Cell<Instant>,
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
            reqs: Cell::new(0),
            remaining: Cell::new(None),
            reset_time: Cell::new(Instant::now()),
        })
    }

    /// Send a request to reddit
    pub fn run_request(&self, req: Request) -> Result<Value> {
        // Ratelimit based on method chosen type
        match self.limit.get() {
            LimitMethod::Steady => {
                // Check if we have a remaining limit
                if let Some(remaining) = self.remaining.get() {
                    // If the reset time is in the future
                    if Instant::now() < self.reset_time.get() {
                        // Sleep for the amount of time until reset divided by how many requests we have for steady sending
                        thread::sleep(
                            (self.reset_time.get() - Instant::now())
                                .checked_div(remaining as u32)
                                .unwrap(),
                        );
                    }
                    // Else we must have already passed reset time and we will get a new one after this request
                }
            }
            LimitMethod::Burst => {
                // Check if we have a remaining limit
                if let Some(remaining) = self.remaining.get() {
                    // If we have none remaining and we haven't passed the request limit, sleep till we do
                    if remaining <= 0 && self.reset_time.get() > Instant::now() {
                        thread::sleep(Instant::now() - self.reset_time.get());
                    }
                }
            }
        };

        // Execute the request!
        let mut response = self.client.execute(req).chain_err(
            || "Failed to send request",
        )?;
        let mut out = String::new();
        response.read_to_string(&mut out).chain_err(|| "Nice")?;

        // Update values from response ratelimiting headers
        if let Some(reqs_used) = response.headers().get_raw("x-ratelimit-used") {
            let reqs_used = String::from_utf8_lossy(reqs_used.one().unwrap())
                .parse::<f32>()
                .unwrap()
                .round() as i32;
            self.reqs.set(reqs_used);
        }
        if let Some(reqs_remaining) = response.headers().get_raw("x-ratelimit-remaining") {
            let reqs_remaining = String::from_utf8_lossy(reqs_remaining.one().unwrap())
                .parse::<f32>()
                .unwrap()
                .round() as i32;
            self.remaining.set(Some(reqs_remaining));
        }
        if let Some(secs_remaining) = response.headers().get_raw("x-ratelimit-reset") {
            let secs_remaining = String::from_utf8_lossy(secs_remaining.one().unwrap())
                .parse::<f32>()
                .unwrap()
                .round() as u64;
            self.reset_time.set(
                Instant::now() +
                    Duration::new(secs_remaining, 0),
            );
        }

        if !response.status().is_success() {
            return Err(ErrorKind::BadRequest(out).into());
        }

        Ok(json::from_str(&out).chain_err(|| "Couldn't parse json")?)
    }

    /// Send a request to reddit with authorization headers
    pub fn run_auth_request(&self, mut req: Request) -> Result<Value> {
        // Check if this connection is authorized
        if let Some(ref auth) = self.auth.clone() {
            req.headers_mut().set(Authorization(
                Bearer { token: auth.token.clone() },
            ));

            self.run_request(req)
        } else {
            Err(ErrorKind::Unauthorized.into())
        }
    }

    pub fn set_limit(&self, limit: LimitMethod) {
        self.limit.set(limit);
    }
}
