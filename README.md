# msg queue use server send event

## 使用
1. install
```bash
cargo install --git https://github.com/yumoh/sse-queue.git
sse-queue -h
```

## action use http 2.0 sse
- put 添加一条消息到指定任务队列
    - url
        - /msg/queue/put
        - /msg/{queue}/put

    - 参数: method: POST
        - queue: str
        - content: body[bytes]
    - 返回:
        - {"code":0, "msg":"ok", "ok":true, "ok":true,"data":true}
        - {"code":1, "msg":"error", "ok":false, "data":false}
- get 从指定任务队列获取最早的一条消息,获取后删除消息
    - url
        - /msg/queue/get
        - /msg/{queue}/get
    - 参数: method: GET
        - queue: str
        - timeout: optional[int] # 超时时间(秒)
    - 返回:
        - {"code":0, "msg":"ok", "ok":true, "ok":true,"data":string}
        - {"code":1, "msg":"error", "ok":false, "data":false}

- pick 从指定任务队列获取最早的一条消息,不删除消息
    - url
        - /msg/queue/pick
        - /msg/{queue}/pick
    - 参数: method: GET
        - queue: str
    - 返回:
        - {"code":0, "msg":"ok", "ok":true, "ok":true,"data":optional[string]}
        - {"code":1, "msg":"error", "ok":false, "data":false}

- last 指定队列中最新的消息
    - url: /msg/{queue}/last
    - 参数: method: GET
        - queue: str
    - 返回:
        - {"code":0, "msg":"ok", "ok":true, "ok":true,"data":optional[string]}
        - {"code":1, "msg":"error", "ok":false, "data":false}

- first 指定队列中最早的消息
    - url: /msg/{queue}/first
    - 参数: method: GET
        - queue: str
    - 返回:
        - {"code":0, "msg":"ok", "ok":true, "ok":true,"data":optional[string]}
        - {"code":1, "msg":"error", "ok":false, "data":false}

- listen 监听一个队列,有消息就及时返回
    - url
        - /msg/queue/listen
        - /msg/{queue}/listen
    - 参数: method: GET
        - queue: str
    - 返回:
        - iter[{"code":0, "msg":"ok", "ok":true,"data":string}]
        - iter[{"code":0, "msg":"iter data","data":string}]
        - iter[{"code":1, "msg":"error", "ok":false, "data":false}]