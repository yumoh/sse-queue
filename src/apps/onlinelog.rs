use std::ops::DerefMut;
use super::auth::TokenAuth;
use super::state::WebCache;
use rocket::tokio::io;
use rocket::{data::ToByteUnit,  post,get, Data, State};


#[post("/upload?<channel>&<name>",data="<data>")]
async fn upload_log_lines1(channel:&str,name:&str, data:Data<'_>, cache: &State<WebCache>,auth: TokenAuth) -> super::WebResult<String>{
    auth.check_pass_root()?;
    let file = cache.open_online_log(channel, name).await?;
    {
        let f = &mut file.lock().await;
        // data.open(4.gibibytes()).stream_to(f).await?;
        io::copy(&mut data.open(4.gibibytes()), f.deref_mut()).await?;
    }
    Ok("ok".to_string())
}


#[post("/upload/<channel>/<name>",data="<data>")]
async fn upload_log_lines2(channel:&str,name:&str, data:Data<'_>, cache: &State<WebCache>,auth: TokenAuth) -> super::WebResult<String>{
    auth.check_pass_root()?;
    let file = cache.open_online_log(channel, name).await?;
    {
        let f = &mut file.lock().await;
        // data.open(4.gibibytes()).stream_to(f).await?;
        io::copy(&mut data.open(4.gibibytes()), f.deref_mut()).await?;
    }
    Ok("ok".to_string())
}

#[get("/close?<channel>&<name>")]
async fn close_log1(channel:&str,name:&str, cache: &State<WebCache>,auth: TokenAuth) -> super::WebResult<String>{
    auth.check_pass_root()?;
    cache.close_online_log(channel, name).await;   
    Ok("ok".to_string())
}

#[get("/close/<channel>/<name>")]
async fn close_log2(channel:&str,name:&str, cache: &State<WebCache>,auth: TokenAuth) -> super::WebResult<String>{
    auth.check_pass_root()?;
    cache.close_online_log(channel, name).await;   
    Ok("ok".to_string())
}

pub fn routes() -> Vec<rocket::Route>{
    rocket::routes![upload_log_lines1,upload_log_lines2,close_log1,close_log2]
}