use std::ops::DerefMut;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use super::auth::TokenAuth;
use super::state::WebCache;
use super::types::ResultBase;
use super::seekstream::FileSeekStream;

use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::tokio::fs;
use rocket::{data::ToByteUnit, get, head, post, Data, State};
use rocket::tokio::io::{AsyncSeekExt, SeekFrom};


/// 1. put
///     - url
///         - "/storage/put"
///         - "/storage/put/{bucket}/{name}"
///     - method: POST
///     - params:
///         - bucket: string, required
///         - name: string, required
///     - request body: any bytes
///     - response json
///         - {"code":1,"msg":"ok","result":true}
///         - {"code":0,"msg":"error","result":false}
#[post("/put?<bucket>&<name>", data = "<data>")]
async fn upload_file1(
    bucket: &str,
    name: &str,
    data: Data<'_>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let mut data_dir = cache.open_data_dir(bucket);
    if !fs::try_exists(&data_dir).await.unwrap_or(false) {
        fs::create_dir_all(&data_dir).await?
    }
    data_dir.push(name);
    let mut file = fs::File::create(data_dir).await?;
    data.open(4.gibibytes()).stream_to(&mut file).await?;
    Ok(Json(ResultBase::ok(true)))
}

/// 1. put
///     - url
///         - "/storage/put"
///         - "/storage/put/{bucket}/{name}"
///     - method: POST
///     - params:
///         - bucket: string, required
///         - name: string, required
///     - request body: any bytes
///     - response json
///         - {"code":1,"msg":"ok","result":true}
///         - {"code":0,"msg":"error","result":false}
#[post("/put/<bucket>/<name>", data = "<data>")]
async fn upload_file2(
    bucket: &str,
    name: &str,
    data: Data<'_>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
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
    Ok(Json(ResultBase::ok(true)))
}

async fn append_file(bucket:&str,name:&str,hold:Option<bool>,data:Data<'_>,cache:&State<WebCache>,auth: TokenAuth)->super::WebResult<Json<ResultBase<bool>>>{
    auth.check_pass_root()?;
    let af = cache.open_append_file(bucket, name).await?;
    let count = {
        let mut afg = af.lock().await;
        if let Some(aft) = (&mut afg.1).deref_mut() {
            data.open(4.gibibytes()).stream_to(aft).await?;
        } else {
            log::warn!("get null file in cache.");
        }
        afg.0.fetch_sub(1,std::sync::atomic::Ordering::Relaxed)
    };
    if count == 0 && !hold.unwrap_or_default() {
        cache.close_append_file(bucket, name).await?;
    }
    // cache.close_append_file(bucket, name).await?;
    Ok(Json(ResultBase::ok(true)))
}

#[post("/append?<bucket>&<name>&<hold>", data = "<data>")]
async fn append_file1(
    bucket: &str,
    name: &str,
    hold:Option<bool>,
    data: Data<'_>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    append_file(bucket,name,hold,data,cache,auth).await
}

#[post("/append/<bucket>/<name>?<hold>", data = "<data>")]
async fn append_file2(
    bucket: &str,
    name: &str,
    hold:Option<bool>,
    data: Data<'_>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    append_file(bucket,name,hold,data,cache,auth).await
}


#[get("/closeappend?<bucket>&<name>")]
async fn close_append_file1(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    cache.close_append_file(bucket, name).await?;
    Ok(Json(ResultBase::ok(true)))
}

#[get("/closeappend/<bucket>/<name>")]
async fn close_append_file2(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    cache.close_append_file(bucket, name).await?;
    Ok(Json(ResultBase::ok(true)))
}

/// 2. get
/// - url
///     - "/storage/get"
///     - "/storage/get/{bucket}/{name}"
/// - method: GET
/// - params:
///     - bucket: string, required
///     - name: string, required
/// - response status: 200 body: any bytes
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

#[head("/get?<bucket>&<name>")]
async fn head_download_file1(
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

/// 2. get
/// - url
///     - "/storage/get"
///     - "/storage/get/{bucket}/{name}"
/// - method: GET
/// - params:
///     - bucket: string, required
///     - name: string, required
/// - response status: 200 body: any bytes
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
    #[cfg(debug_assertions)]
    {
        log::info!("headers: {:?}",&headers.kv);
    }
    if let Some(x) = headers.kv.get("range") {
        let (ranges, errors) = range_header::ByteRange::parse(x)
            .iter()
            .map(super::seekstream::range_header_parts)
            .map(|(start, end)| super::seekstream::to_satisfiable_range(start, end, content_len))
            .partition::<Vec<_>, _>(|x| x.is_ok());

        if !errors.is_empty() || ranges.is_empty() {
            for e in errors {
                log::warn!("{:?}", e.unwrap_err());
            }
            return Err(super::WebError::new("range parameter error"));
        }
        let mut ranges: Vec<(u64, u64)> = ranges.iter().map(|x| x.unwrap()).collect();
        // de-duplicate the list of ranges
        ranges.sort();
        ranges.dedup_by(|&mut (a, b), &mut (c, d)| a == c && b == d);

        // Stream multipart/bytes if multiple ranges have been requested
        if ranges.len() > 1 {
            Err(super::WebError::new("not support multipart ranges"))
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
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let result = ResultBase::ok(fs::metadata(path).await.is_ok());
    Ok(Json(result))
}

#[get("/exists/<bucket>/<name>")]
async fn exists_file2(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let result = ResultBase::ok(fs::metadata(path).await.is_ok());
    Ok(Json(result))
}


#[get("/fsize?<bucket>&<name>")]
async fn fsize_file1(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<u64>>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let fsize = fs::metadata(path).await?.size();
    let result = ResultBase::ok(fsize);
    Ok(Json(result))
}

#[get("/fsize/<bucket>/<name>")]
async fn fsize_file2(
    bucket: &str,
    name: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<u64>>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let fsize = fs::metadata(path).await?.size();
    let result = ResultBase::ok(fsize);
    Ok(Json(result))
}


/// 3. create
///     - url
///         - "/storage/new"
///         - "/storage/new/{bucket}"
///     - method: GET
///     - params:
///         - bucket: string, required
///     - response json
///         - {"code":1,"msg":"ok","result":true}
///         - {"code":0,"msg":"error","result":false}  
#[get("/new?<bucket>")]
async fn create_bucket1(
    bucket: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let data_dir = cache.open_data_dir(bucket);
    if !fs::try_exists(&data_dir).await.unwrap_or(false) {
        fs::create_dir_all(&data_dir).await?
    }
    Ok(Json(ResultBase::ok(true)))
}
/// 3. create
///     - url
///         - "/storage/new"
///         - "/storage/new/{bucket}"
///     - method: GET
///     - params:
///         - bucket: string, required
///     - response json
///         - {"code":1,"msg":"ok","result":true}
///         - {"code":0,"msg":"error","result":false}
#[get("/new/<bucket>")]
async fn create_bucket2(
    bucket: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let data_dir = cache.open_data_dir(bucket);
    if !fs::try_exists(&data_dir).await.unwrap_or(false) {
        fs::create_dir_all(&data_dir).await?
    }
    Ok(Json(ResultBase::ok(true)))
}

async fn list_bucket(parent:impl AsRef<Path>,filter:Option<&str>)-> super::WebResult<Vec<String>>{
    let mut entries = fs::read_dir(parent.as_ref()).await?;
    let mut file_list = vec![];
    if let Some(filter) = filter {
        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with(filter) {
                    file_list.push(file_name.to_string());
                }
            }
        }
    } else {
        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                file_list.push(file_name.to_string());
            }
        }
    }
    Ok(file_list)
}

#[get("/list?<bucket>&<filter>")]
async fn list_bucket1(
    bucket: &str,
    filter: Option<&str>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<Vec<String>>>> {
    auth.check_pass_root()?;
    let data_dir = cache.open_data_dir(bucket);
    let file_list = list_bucket(&data_dir, filter).await?;
    Ok(Json(ResultBase::ok(file_list)))
}

#[get("/list/<bucket>?<filter>")]
async fn list_bucket2(
    bucket: &str,
    filter: Option<&str>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<Vec<String>>>> {
    auth.check_pass_root()?;
    let data_dir = cache.open_data_dir(bucket);
    let file_list = list_bucket(&data_dir, filter).await?;
    Ok(Json(ResultBase::ok(file_list)))
}

// #[get("/del?<bucket>")]
// async fn delete_bucket1(
//     bucket: &str,
//     auth: TokenAuth,
//     cache: &State<WebCache>,
// ) -> super::WebResult<Json<ResultBase<bool>>> {
//     auth.check_pass_root()?;
//     let path = cache.open_data_dir(bucket);
//     fs::remove_dir_all(path).await?;
//     Ok(Json(ResultBase::ok(true)))
// }

#[get("/del/<bucket>")]
async fn delete_bucket2(
    bucket: &str,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let path = cache.open_data_dir(bucket);
    fs::remove_dir_all(path).await?;
    Ok(Json(ResultBase::ok(true)))
}

#[get("/del/<bucket>/<name>?<exists_ok>")]
async fn delete_file3(
    bucket: &str,
    name: &str,
    exists_ok: Option<bool>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let mut path = cache.open_data_dir(bucket);
    path.push(name);
    let exists_ok = exists_ok.unwrap_or(false);
    if !exists_ok {
        fs::remove_file(path).await?;
    } else {
        let _ = fs::remove_file(path).await;
    }
    Ok(Json(ResultBase::ok(true)))
}

#[get("/del?<bucket>&<name>&<exists_ok>")]
async fn delete_file4(
    bucket: &str,
    name: Option<&str>,
    exists_ok: Option<bool>,
    auth: TokenAuth,
    cache: &State<WebCache>,
) -> super::WebResult<Json<ResultBase<bool>>> {
    auth.check_pass_root()?;
    let exists_ok = exists_ok.unwrap_or(false);
    if let Some(name) = name {
        let mut path = cache.open_data_dir(bucket);
        path.push(name);
        if !exists_ok {
            fs::remove_file(path).await?;
        } else {
            let _ = fs::remove_file(path).await;
        }
    } else {
        let path = cache.open_data_dir(bucket);
        if !exists_ok {
            fs::remove_dir_all(path).await?;
        } else {
            let _ = fs::remove_dir_all(path).await;
        }
        
    }
    Ok(Json(ResultBase::ok(true)))
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        upload_file1,
        upload_file2,
        download_file1,
        download_file2,
        head_download_file1,
        head_download_file2,
        list_bucket1,
        list_bucket2,
        exists_file1,
        exists_file2,
        create_bucket1,
        create_bucket2,
        // delete_bucket1,
        delete_bucket2,
        delete_file3,
        delete_file4,
        append_file1,
        append_file2,
        close_append_file1,
        close_append_file2,
        fsize_file1,
        fsize_file2,
    ]
}
