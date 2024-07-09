use super::auth::TokenAuth;
use super::state::WebCache;
use rocket::serde::json::Json;
use rocket::tokio::io::AsyncReadExt;
use rocket::{data::ToByteUnit, get, post, Data, State};

/// {"code":1, "msg":"ok","result":true}
#[derive(rocket::serde::Serialize)]
#[serde(crate = "rocket::serde")]
struct ResultPut {
    code: u8,
    msg: String,
    result: bool,
}

impl ResultPut {
    fn ok() -> Self {
        Self {
            code: 1,
            msg: "ok".to_string(),
            result: true,
        }
    }
}
/// - {"code":1, "msg":"ok","result":true,"content":body[bytes]}
/// - {"code":0, "msg":"error","result":false}
#[derive(rocket::serde::Serialize)]
#[serde(crate = "rocket::serde")]
struct ResultGet {
    code: u8,
    msg: String,
    result: bool,
    content: Option<String>,
}

impl ResultGet {
    fn ok(content: Option<Vec<u8>>) -> Self {
        if let Some(data) = content {
            Self {
                code: 1,
                msg: "ok".to_string(),
                result: true,
                content: Some(String::from_utf8_lossy(&data).to_string()),
            }
        } else {
            Self {
                code: 1,
                msg: "empty".to_string(),
                result: false,
                content: None,
            }
        }
    }
    #[allow(unused)]
    fn err(err: impl ToString) -> Self {
        Self {
            code: 0,
            msg: err.to_string(),
            result: false,
            content: None,
        }
    }
}
#[post("/queue/put?<queue>", data = "<data>")]
async fn put_to_queue1<'r>(
    queue: &str,
    data: Data<'r>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let mut bytes = Vec::new();
    // 限制大小为10M
    if data
        .open(10.mebibytes())
        .read_to_end(&mut bytes)
        .await
        .is_err()
    {
        return Err(super::WebError::new("data too large"));
    }
    state.queue_push_msg(queue, bytes).await;
    Ok(Json(ResultPut::ok()))
}

#[post("/<queue>/put", data = "<data>")]
async fn put_to_queue2<'r>(
    queue: &str,
    data: Data<'r>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    // log::debug!("auth pass");
    let mut bytes = Vec::new();
    // 限制大小为10M
    if data
        .open(10.mebibytes())
        .read_to_end(&mut bytes)
        .await
        .is_err()
    {
        return Err(super::WebError::new("data too large"));
    }
    // log::debug!("data recviced. {}",String::from_utf8_lossy(&bytes));
    state.queue_push_msg(queue, bytes).await;
    // log::debug!("queue pushed.");
    Ok(Json(ResultPut::ok()))
}

#[get("/<queue>/put?<content>")]
async fn put_to_queue3<'r>(
    queue: &str,
    content: String,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    state.queue_push_msg(queue, content.as_bytes().to_vec()).await;
    Ok(Json(ResultPut::ok()))
}

async fn wait_msg(queue: &str, timeout: Option<usize>, state: &State<WebCache>) -> Option<Vec<u8>> {
    let mut msg = state.queue_pop_msg(queue).await;
    if msg.is_none() {
        if let Some(timeout) = timeout {
            let xtimeout = std::time::Duration::from_secs(timeout as u64);
            let xstart = std::time::Instant::now();
            while msg.is_none() && xtimeout > (std::time::Instant::now() - xstart) {
                msg = state.queue_pop_msg(queue).await;
                rocket::tokio::time::sleep(std::time::Duration::from_secs_f32(0.1)).await;
            }
        }
    }
    msg
}

#[get("/queue/get?<queue>&<timeout>")]
async fn pop_from_queue1<'r>(
    queue: &str,
    timeout: Option<usize>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultGet>> {
    auth.check_pass_root()?;
    let msg = wait_msg(queue, timeout, state).await;
    Ok(Json(ResultGet::ok(msg)))
}

#[get("/<queue>/get?<timeout>")]
async fn pop_from_queue2<'r>(
    queue: &str,
    timeout: Option<usize>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultGet>> {
    auth.check_pass_root()?;
    let msg = wait_msg(queue, timeout, state).await;
    Ok(Json(ResultGet::ok(msg)))
}

#[get("/queue/pick?<queue>&<index>")]
async fn pick_from_queue1<'r>(
    queue: &str,
    index: usize,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultGet>> {
    auth.check_pass_root()?;
    let msg = state.queue_pick_msg(queue, index).await;
    Ok(Json(ResultGet::ok(msg)))
}

#[get("/<queue>/pick/<index>")]
async fn pick_from_queue2<'r>(
    queue: &str,
    index: usize,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultGet>> {
    auth.check_pass_root()?;
    let msg = state.queue_pick_msg(queue, index).await;
    Ok(Json(ResultGet::ok(msg)))
}

#[get("/<queue>/last")]
async fn last_from_queue<'r>(
    queue: &str,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultGet>> {
    auth.check_pass_root()?;
    let msg = state.queue_last(queue).await;
    Ok(Json(ResultGet::ok(msg)))
}

#[get("/<queue>/first")]
async fn first_from_queue<'r>(
    queue: &str,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultGet>> {
    auth.check_pass_root()?;
    let msg = state.queue_first(queue).await;
    Ok(Json(ResultGet::ok(msg)))
}

use rocket::response::stream::{Event, EventStream};
#[get("/queue/listen?<queue>")]
async fn listen_from_queue1<'r>(
    queue: &'r str,
    state: &'r State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<EventStream![Event + 'r]> {
    auth.check_pass_root()?;
    Ok(EventStream! {
        let mut interval = rocket::tokio::time::interval(std::time::Duration::from_secs_f32(0.1));
        loop {
            let msg = state.queue_pop_msg(queue).await;
            if msg.is_some() {
                yield Event::json(&ResultGet::ok(msg));
            } else {
                yield Event::empty();
                interval.tick().await;
            }
        }
    })
}

#[get("/<queue>/listen")]
async fn listen_from_queue2<'r>(
    queue: &'r str,
    state: &'r State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<EventStream![Event + 'r]> {
    auth.check_pass_root()?;
    Ok(EventStream! {
        let mut interval = rocket::tokio::time::interval(std::time::Duration::from_secs_f32(0.1));
        loop {
            let msg = state.queue_pop_msg(queue).await;
            if msg.is_some() {
                yield Event::json(&ResultGet::ok(msg));
            } else {
                yield Event::empty();
                interval.tick().await;
            }
        }
    })
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        put_to_queue1,
        put_to_queue2,
        put_to_queue3,
        pop_from_queue1,
        pop_from_queue2,
        pick_from_queue1,
        pick_from_queue2,
        listen_from_queue1,
        listen_from_queue2,
        last_from_queue,
        first_from_queue,
    ]
}
