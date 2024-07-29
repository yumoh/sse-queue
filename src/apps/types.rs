
use rocket::serde::Serialize;
/// {"code":1, "msg":"ok","result":true}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub(super) struct ResultBase<T: Serialize> {
    // code: u8 
    // 0 表示正常
    // > 0 表示错误码
    code: u8,
    // true 正常 false 异常
    ok:  bool,
    // 正常或异常的描述
    msg: String,
    data: T,
}

impl<T> ResultBase<T> where T: Serialize {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            ok: true,
            msg: "ok".to_string(),
            data,
        }
    }
    #[allow(unused)]
    pub fn err(mut self, msg: impl ToString) -> Self {
        self.msg = msg.to_string();
        self.ok = false;
        self.code = 1;
        self
    }
    #[allow(unused)]
    pub fn data(mut self, data: T) -> Self {
        self.data = data;
        self.ok = true;
        self.code = 0;
        self
    }
}