use super::auth::TokenAuth;
use super::state::WebCache;
use rocket::fs::NamedFile;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use rocket::tokio::fs;
use rocket::{data::ToByteUnit, get, head, http, post, Data, Request, State};
use rocket::tokio::io::{AsyncSeekExt, SeekFrom};

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
struct FileSeekStream {
    content_len: u64,
    range1: Option<(u64, u64)>,
    fp: fs::File,
}

impl FileSeekStream {
    pub fn perform_stream(self) -> rocket::Response<'static> {
        let mut resp = rocket::Response::build();
        resp.header(http::ContentType::Binary);
        resp.raw_header("Accept-Ranges", "bytes");
        let res = resp
            .sized_body(self.content_len as usize, self.fp)
            .finalize();
        res
    }
    pub fn perform_range1(self,start:u64,end:u64) -> rocket::Response<'static> {
            // Seek the stream to the desired position
            let range_len = (end + 1 - start) as usize;
            let mut resp = rocket::Response::build();
            resp.header(http::ContentType::Binary);
            resp.raw_header(
                "Content-Range",
                format!("bytes {}-{}/{}", start, end, self.content_len),
            );
            // Set the content length to be the length of the partial stream
            resp.raw_header("Content-Length", format!("{}", range_len));
            resp.status(rocket::http::Status::PartialContent);
            let res = if end + 1 < self.content_len {
                // let tfp = self.fp.take(end + 1 - start);
                resp
                .sized_body(range_len, self.fp)
                .finalize()
            } else {
                resp
                .sized_body(range_len, self.fp)
                .finalize()
            };
            res
    }
}
#[rocket::async_trait]
impl<'r> Responder<'r, 'r> for FileSeekStream {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'r> {
        let res = if let Some((start,end)) = self.range1 {
            self.perform_range1(start, end)
        } else {
            self.perform_stream()
        };
        Ok(res)
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
) -> super::WebResult<NamedFile> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    Ok(NamedFile::open(path).await?)
}

// #[head("/get?<bucket>&<name>")]
// async fn head_download_file1(
//     bucket: &str,
//     name: &str,
//     auth: TokenAuth,
//     cache: &State<WebCache>,
// ) -> super::WebResult<FileSeekStream> {
//     auth.check_pass_root()?;
//     let mut path = cache.open_data_dir(bucket);
//     path.push(name);
//     let mut resp = rocket::Response::build();
//     resp.header(http::ContentType::Binary);
//     resp.header(http::Header::new("Accept-Ranges", "bytes"));
//     let content_len = fs::metadata(&path).await?.len();
//     resp.header(http::Header::new("Content-Length", format!("{content_len}")));
//     let fp = fs::File::open(path).await?;
//     let fileseek = FileSeekStream {content_len,fp};
//     Ok(fileseek)
// }

#[get("/get/<bucket>/<name>")]
async fn download_file2<'r>(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
    headers: super::auth::Headers,
) -> super::WebResult<FileSeekStream> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let content_len = fs::metadata(&path).await?.len();
    let mut fp = fs::File::open(path).await?;

    if let Some(x) = headers.kv.get("Range") {
        let (ranges, errors) = range_header::ByteRange::parse(x)
            .iter()
            .map(super::seekstream::range_header_parts)
            .map(|(start, end)| super::seekstream::to_satisfiable_range(start, end, content_len))
            .partition::<Vec<_>, _>(|x| x.is_ok());

        // If any of the ranges produce an incorrect value,
        // Or the list of ranges is empty.
        // Return a range error.
        if !errors.is_empty() || ranges.is_empty() {
            for e in errors {
                log::warn!("{:?}", e.unwrap_err());
            }
            return Err(super::WebError::new("range parameter error"));
        }

        // Unwrap all the results
        let mut ranges: Vec<(u64, u64)> = ranges.iter().map(|x| x.unwrap()).collect();

        // de-duplicate the list of ranges
        ranges.sort();
        ranges.dedup_by(|&mut (a, b), &mut (c, d)| a == c && b == d);

        // Stream multipart/bytes if multiple ranges have been requested
        if ranges.len() > 1 {
            panic!("not support");
        } else {
            // Stream a single range request if only one was present in the byte ranges
            let &(start, end) = ranges.first().unwrap();
            fp.seek(SeekFrom::Start(start)).await?;
            let fileseek = FileSeekStream { content_len, range1:Some((start,end)),fp };
            Ok(fileseek)
        }
    } else {
        let fileseek = FileSeekStream { content_len, range1:None,fp };
        Ok(fileseek)
    }

}

#[head("/get/<bucket>/<name>")]
async fn head_download_file2(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<FileSeekStream> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let content_len = fs::metadata(&path).await?.len();
    let fp = fs::File::open(path).await?;
    let fileseek = FileSeekStream { content_len,range1:None, fp };
    Ok(fileseek)
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

#[get("/new?<bucket>")]
async fn craete_bucket1(
    bucket: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let data_dir = cache.open_data_dir(bucket);
    if !fs::try_exists(&data_dir).await.unwrap_or(false) {
        fs::create_dir_all(&data_dir).await?
    }
    Ok(Json(ResultPut::ok()))
}

#[get("/new/<bucket>")]
async fn crate_bucket2(
    bucket: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultPut>> {
    auth.check_pass_root()?;
    let data_dir = cache.open_data_dir(bucket);
    if !fs::try_exists(&data_dir).await.unwrap_or(false) {
        fs::create_dir_all(&data_dir).await?
    }
    Ok(Json(ResultPut::ok()))
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
        // head_download_file1,
        head_download_file2,
        exists_file1,
        exists_file2,
        craete_bucket1,
        crate_bucket2,
        delete_bucket1,
        delete_bucket2,
        delete_file3,
    ]
}
