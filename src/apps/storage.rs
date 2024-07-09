use super::auth::TokenAuth;
use super::state::WebCache;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::tokio::fs;
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
    #[allow(unused)]
    pub fn with_msg(mut self, msg: impl ToString) -> Self {
        self.msg = msg.to_string();
        self
    }
    pub fn with_result(mut self, result: bool) -> Self {
        self.result = result;
        self
    }
}

#[post("/put?<bucket>&<name>", data = "<data>")]
async fn upload_file1(
    bucket: &str,
    name: &str,
    data: Data<'_>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let mut data_dir = cache.open_data_dir(bucket);
    if !fs::try_exists(&data_dir).await.unwrap_or(false) {
        fs::create_dir_all(&data_dir).await?
    }
    // if fs::metadata(&data_dir).await.is_err() {
    //     fs::create_dir_all(&data_dir).await?;
    // }
    data_dir.push(name);
    let mut file = fs::File::create(data_dir).await?;
    data.open(4.gibibytes()).stream_to(&mut file).await?;
    Ok(Json(ResultPut::ok()))
}

#[post("/put/<bucket>/<name>", data = "<data>")]
async fn upload_file2(
    bucket: &str,
    name: &str,
    data: Data<'_>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let mut data_dir = cache.open_data_dir(bucket);
    if !fs::try_exists(&data_dir).await.unwrap_or(false) {
        fs::create_dir_all(&data_dir).await?
    }
    // if fs::metadata(&data_dir).await.is_err() {
    //     fs::create_dir_all(&data_dir).await?;
    // }
    data_dir.push(name);
    // log::debug!("save file at: {}",data_dir.display());
    let mut file = fs::File::create(data_dir).await?;
    data.open(4.gibibytes()).stream_to(&mut file).await?;
    Ok(Json(ResultPut::ok()))
}

#[get("/get?<bucket>&<name>")]
async fn download_file1(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Option<NamedFile>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    Ok(NamedFile::open(path).await.ok())
}

#[get("/get/<bucket>/<name>")]
async fn download_file2(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Option<NamedFile>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    Ok(NamedFile::open(path).await.ok())
}

#[get("/exists?<bucket>&<name>")]
async fn exists_file1(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let result = ResultPut::ok().with_result(fs::metadata(path).await.is_ok());
    Ok(Json(result))
}

#[get("/exists/<bucket>/<name>")]
async fn exists_file2(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let result = ResultPut::ok().with_result(fs::metadata(path).await.is_ok());
    Ok(Json(result))
}

#[get("/del?<bucket>")]
async fn delete_bucket1(
    bucket: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let path = cache.open_data_dir(bucket);
    fs::remove_dir_all(path).await?;
    Ok(Json(ResultPut::ok()))
}

#[get("/del/<bucket>")]
async fn delete_bucket2(
    bucket: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let path = cache.open_data_dir(bucket);
    fs::remove_dir_all(path).await?;
    Ok(Json(ResultPut::ok()))
}

#[get("/del/<bucket>/<name>")]
async fn delete_file3(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    fs::remove_file(path).await?;
    Ok(Json(ResultPut::ok()))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        upload_file1,
        upload_file2,
        download_file1,
        download_file2,
        exists_file1,
        exists_file2,
        delete_bucket1,
        delete_bucket2,
        delete_file3,
    ]
}
