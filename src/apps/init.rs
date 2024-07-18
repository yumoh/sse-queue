
use rocket::get;
use rocket::request::{FromRequest, Outcome, Request};
use chrono::{
    prelude::{Local, Timelike},
    Datelike,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimeData {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    microsecond: u32,
}

impl Default for TimeData {
    fn default() -> Self {
        let now = Local::now();
        TimeData {
            year: now.year(),
            month: now.month(),
            day: now.day(),
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
            microsecond: now.nanosecond(),
        }
    }
}

impl std::fmt::Display for TimeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} ",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

#[derive(Debug)]
pub struct IpAddrHeader(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IpAddrHeader {
    type Error = super::WebError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = req.headers();
        let connect_ip = req.client_ip().map(|ip| ip.to_string());
        let ip = headers
            .get_one("x-real-ip")
            .or(headers.get_one("x-forwarded-for"))
            .map(|s| s.to_string())
            .or(connect_ip);
        match ip {
            Some(ip) => Outcome::Success(IpAddrHeader(ip)),
            None => Outcome::Error((
                rocket::http::Status::BadRequest,
                super::WebError::from("ip not found"),
            )),
        }
    }
}

#[get("/")]
async fn index() -> &'static str {
    "sse event queue"
}

#[get("/ping")]
async fn echo_ping() -> &'static str {
    "pong"
}

#[get("/version")]
async fn echo_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[get("/commit-id")]
async fn echo_commit_id() -> &'static str {
    env!("GIT_HASH")
}

#[get("/time")]
async fn echo_time() -> String {
    let now = TimeData::default();
    format!("{}", &now)
}

#[get("/ip")]
async fn echo_ip(client_ip: IpAddrHeader) -> String {
    client_ip.0.to_string()
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![index, echo_ping, echo_version,echo_commit_id, echo_time, echo_ip]
}  