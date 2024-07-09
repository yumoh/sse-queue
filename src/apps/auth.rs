use super::state::WebCache;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::{http::Status, State};

pub struct TokenAuth {
    /// 参数或 header/url query 中的 token
    #[allow(unused)]
    request_token: Option<String>,
    /// root token 是否已经通过认证
    pass_root_token: bool,
    /// 是否需要认证
    /// 如果配置了root token或者配置了app token，则需要认证
    need_auth: bool,
}

impl TokenAuth {
    /// 检查是否已经通过root token
    pub fn pass_root(&self) -> bool {
        if !self.need_auth {
            true
        } else {
            self.pass_root_token
        }
    }
    /// 检查是否已经通过root token
    pub fn check_pass_root(&self) -> std::result::Result<(), String> {
        if !self.pass_root() {
            Err("token error".to_string())
        } else {
            Ok(())
        }
    }
    // 检查 app config 中的 token 认证
    // pub fn check_token(&self, token: Option<&String>) -> super::WebResult<()> {
    //     // root token 已经通过认证
    //     if self.pass_root_token {
    //         return Ok(());
    //     }
    //     // 如果 app config 中配置了 token，则需要root token认证或者app config 中配置的token认证
    //     if let Some(tc) = token {
    //         if let Some(t) = &self.request_token {
    //             if t == tc {
    //                 return Ok(());
    //             }
    //         }
    //         log::warn!("token error: {:?}", self.request_token);
    //         Err(super::WebError::from("token error"))
    //     } else if self.need_auth {
    //         if let Some(token) = self.request_token.as_ref() {
    //             log::warn!("token error. get error token {}", token);
    //         }
    //         Err(super::WebError::from("token error"))
    //     } else {
    //         Ok(())
    //     }
    // }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenAuth {
    type Error = super::WebError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut token = req.headers().get("_token").next().map(|v| v.to_string());
        if token.is_none() {
            token = req
                .query_value::<&str>("_token").or(req.query_value::<&str>("token"))
                .map(|v| v.unwrap().to_string());
        }

        let mut pass_root_token = false;
        let mut need_auth = false;
        let state = req.guard::<&State<WebCache>>().await;
        
        if let Outcome::Success(state) = state {
            if let Some(tp) = state.token.as_ref() {
                need_auth = true;
                if matches!(&token, Some(t) if t == tp.as_ref()) {
                    pass_root_token = true;
                } else if req.uri().path().ends_with("/admin") {
                    return Outcome::Error((
                        Status::NonAuthoritativeInformation,
                        super::WebError::from("admin token error."),
                    ));
                }
            }
        }
        let auth = TokenAuth {
            request_token: token,
            pass_root_token,
            need_auth,
        };
        Outcome::Success(auth)
    }
}

// pub struct Headers {
//     pub kv: Vec<(String, String)>,
// }

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for Headers {
//     type Error = super::WebError;

//     async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
//         let mut kv = req
//             .headers()
//             .iter()
//             .map(|item| (item.name().to_string(), item.value().to_string()))
//             .collect::<Vec<_>>();
//         kv.retain(|(k, _)| k != "_token");
//         let kvs = Headers { kv };
//         Outcome::Success(kvs)
//     }
// }
