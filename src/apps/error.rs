use std::fmt::Display;

#[derive(Debug)]
pub enum WebError {
    Io(std::io::Error),
    Other(String),
    // Timeout,
}

impl Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebError::Io(err) => write!(f, "io error: {}", err),
            WebError::Other(err) => write!(f, "other error: {}", err),
            // WebError::Timeout => write!(f, "timeout error"),
        }
    }
}

impl std::error::Error for WebError {}

pub type WebResult<T> = std::result::Result<T, WebError>;

impl From<String> for WebError {
    fn from(err: String) -> Self {
        Self::Other(err)
    }
}

impl From<&str> for WebError {
    fn from(err: &str) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<serde_json::Error> for WebError {
    fn from(value: serde_json::Error) -> Self {
        Self::Other(format!("serde_json error: {}", value))
    }
}

// impl From<reqwest::Error> for AppError {
//     fn from(err: reqwest::Error) -> Self {
//         Self::Other(format!("reqwest error: {}", err))
//     }
// }

impl From<toml::de::Error> for WebError {
    fn from(err: toml::de::Error) -> Self {
        Self::Other(format!("toml decode err: {err:?}"))
    }
}
impl From<toml::ser::Error> for WebError {
    fn from(err: toml::ser::Error) -> Self {
        Self::Other(format!("toml encode err: {err:?}"))
    }
}

impl From<std::io::Error> for WebError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl WebError {
    pub fn new(err: impl ToString) -> Self {
        Self::Other(err.to_string())
    }
}

/// {"code":1, "msg":"ok","result":true}
#[derive(rocket::serde::Serialize)]
#[serde(crate = "rocket::serde")]
struct ResultError {
    code: u8,
    msg: String,
    r#ok: bool,
}

impl ResultError {
    fn err(err: impl ToString) ->  Self {
        Self {
            code: 1,
            msg: err.to_string(),
            r#ok: false,
        }
    }
}

use rocket::{
    http,
    response::{self, Responder, Response},
    Request,
};

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for super::error::WebError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let body = serde_json::to_string(&ResultError::err(self)).unwrap();
        log::warn!("error occurrs: {body}");
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .status(http::Status::InternalServerError)
            .ok()
    }
}
