


// Convert a range to a satisfiable range
pub(super) fn to_satisfiable_range(
    from: Option<u64>,
    to: Option<u64>,
    length: u64,
) -> Result<(u64, u64), &'static str> {
    let (start, mut end) = match (from, to) {
        (Some(x), Some(z)) => (x, z),                // FromToAll
        (Some(x), None) => (x, length - 1),          // FromTo
        (None, Some(z)) => (length - z, length - 1), // FromEnd
        (None, None) => return Err("You need at least one value to satisfy a range request"),
    };

    if end < start {
        return Err("A byte-range-spec is invalid if the last-byte-pos value is present and less than the first-byte-pos.");
    }
    if end > length {
        end = length
    }

    Ok((start, end))
}

pub(super) fn range_header_parts(header: &range_header::ByteRange) -> (Option<u64>, Option<u64>) {
    use range_header::ByteRange::{FromTo, FromToAll, Last};
    match *header {
        FromTo(x) => (Some(x), None),
        FromToAll(x, y) => (Some(x), Some(y)),
        Last(x) => (None, Some(x)),
    }
}

use rocket::tokio::fs;
use rocket::{http,Request,response::{self,Responder}};

pub(super) struct FileSeekStream {
    pub(super)content_len: u64,
    pub(super)range1: Option<(u64, u64)>,
    pub(super)fp: fs::File,
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
            // resp.raw_header("Content-Length", format!("{}", range_len));
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