use super::auth::TokenAuth;
use super::state::WebCache;
use super::types::ResultBase;
use rocket::serde::json::Json;
use rocket::tokio::io::AsyncReadExt;
use rocket::{data::ToByteUnit, get, post, Data, State};

#[post("/queue/put?<queue>", data = "<data>")]
async fn put_to_queue1<'r>(
    queue: &str,
    data: Data<'r>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<bool>>> {
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
    Ok(Json(ResultBase::ok(true)))
}

#[post("/<queue>/put", data = "<data>")]
async fn put_to_queue2<'r>(
    queue: &str,
    data: Data<'r>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<bool>>> {
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
    Ok(Json(ResultBase::ok(true)))
}

#[get("/<queue>/put?<content>")]
async fn put_to_queue3<'r>(
    queue: &str,
    content: String,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    state.queue_push_msg(queue, content.as_bytes().to_vec()).await;
    Ok(Json(ResultBase::ok(true)))
}

async fn wait_msg(queue: &str, timeout: Option<usize>, state: &State<WebCache>) -> Option<String> {
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
    msg.map(|s|String::from_utf8_lossy(&s).to_string())
}

#[get("/queue/get?<queue>&<timeout>")]
async fn pop_from_queue1<'r>(
    queue: &str,
    timeout: Option<usize>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<Option<String>>>> {
    auth.check_pass_root()?;
    let msg = wait_msg(queue, timeout, state).await;
    Ok(Json(ResultBase::ok(msg)))
}

#[get("/<queue>/get?<timeout>")]
async fn pop_from_queue2<'r>(
    queue: &str,
    timeout: Option<usize>,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<Option<String>>>> {
    auth.check_pass_root()?;
    let msg = wait_msg(queue, timeout, state).await;
    Ok(Json(ResultBase::ok(msg)))
}

#[get("/queue/pick?<queue>&<index>")]
async fn pick_from_queue1<'r>(
    queue: &str,
    index: usize,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<Option<String>>>> {
    auth.check_pass_root()?;
    let msg = state.queue_pick_msg(queue, index).await;
    Ok(Json(ResultBase::ok(msg.map(|s|String::from_utf8_lossy(&s).to_string()))))
}

#[get("/<queue>/pick/<index>")]
async fn pick_from_queue2<'r>(
    queue: &str,
    index: usize,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<Option<String>>>> {
    auth.check_pass_root()?;
    let msg = state.queue_pick_msg(queue, index).await;
    Ok(Json(ResultBase::ok(msg.map(|s|String::from_utf8_lossy(&s).to_string()))))
}

#[get("/<queue>/last")]
async fn last_from_queue<'r>(
    queue: &str,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<Option<String>>>> {
    auth.check_pass_root()?;
    let msg = state.queue_last(queue).await;
    Ok(Json(ResultBase::ok(msg.map(|s|String::from_utf8_lossy(&s).to_string()))))
}

#[get("/<queue>/first")]
async fn first_from_queue<'r>(
    queue: &str,
    state: &State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<Json<ResultBase<Option<String>>>> {
    auth.check_pass_root()?;
    let msg = state.queue_first(queue).await;
    Ok(Json(ResultBase::ok(msg.map(|s|String::from_utf8_lossy(&s).to_string()))))
}

use rocket::response::stream::{Event, EventStream};
#[get("/queue/listen?<queue>&<timeout>")]
async fn listen_from_queue1<'r>(
    queue: &'r str,
    timeout: Option<u64>,
    state: &'r State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<EventStream![Event + 'r]> {
    auth.check_pass_root()?;
    let queue = state.queue_listen(queue).await;
    let inst_until = timeout.map(|v| std::time::Instant::now() + std::time::Duration::from_secs(v));
    Ok(EventStream! {
        let mut interval = rocket::tokio::time::interval(std::time::Duration::from_secs_f32(0.1));
        loop {
            let msg = queue.lock().await.pop_back();
            if msg.is_some() {
                let msg = msg.map(|s|String::from_utf8_lossy(&s).to_string());
                yield Event::json(&ResultBase::ok(msg));
            } else {
                // yield Event::empty();
                if let Some(until) = &inst_until {
                    if &std::time::Instant::now() > until {
                        log::info!("timeout to close sse");
                        yield Event::data("bye");
                        break;
                    }
                } else {
                    interval.tick().await;
                }
            }
        }
    })
}

#[get("/<queue>/listen?<timeout>")]
async fn listen_from_queue2<'r>(
    queue: &'r str,
    timeout: Option<u64>,
    state: &'r State<WebCache>,
    auth: TokenAuth,
) -> super::WebResult<EventStream![Event + 'r]> {
    auth.check_pass_root()?;
    let queue = state.queue_listen(queue).await;
    let inst_until = timeout.map(|v| std::time::Instant::now() + std::time::Duration::from_secs(v));
    Ok(EventStream! {
        let mut interval = rocket::tokio::time::interval(std::time::Duration::from_secs_f32(0.1));
        loop {
            let msg = queue.lock().await.pop_back();
            if msg.is_some() {
                let msg = msg.map(|s|String::from_utf8_lossy(&s).to_string());
                yield Event::json(&ResultBase::ok(msg));
            } else {
                // yield Event::empty();
                if let Some(until) = &inst_until {
                    if &std::time::Instant::now() > until {
                        log::info!("timeout to close sse");
                        yield Event::data("bye");
                        break;
                    }
                } else {
                    interval.tick().await;
                }
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
