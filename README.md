# msg queue use server send event

# action use http 2.0 sse
- put 添加一条消息到指定任务队列
    - url
        - /msg/queue/put
        - /msg/{queue}/put

    - 参数: method: POST
        - queue: str
        - content: body[bytes]
    - 返回:
        - {"code":1, "msg":"ok","result":true}
        - {"code":0, "msg":"error","result":false}
- get 从指定任务队列获取最早的一条消息,获取后删除消息
    - url
        - /msg/queue/get
        - /msg/{queue}/get
    - 参数: method: GET
        - queue: str
        - timeout: optional[int] # 超时时间(秒)
    - 返回:
        - {"code":1, "msg":"ok","result":true,"content":body[bytes]}
        - {"code":0, "msg":"error","result":false}

- pick 从指定任务队列获取最早的一条消息,不删除消息
    - url
        - /msg/queue/pick
        - /msg/{queue}/pick
    - 参数: method: GET
        - queue: str
    - 返回:
        - {"code":1, "msg":"ok","result":true,"content":body[bytes]}
        - {"code":0, "msg":"error","result":false}

- listen 监听一个队列,有消息就及时返回
    - url
        - /msg/queue/listen
        - /msg/{queue}/listen
    - 参数: method: GET
        - queue: str
    - 返回:
        - iter[{"code":1, "msg":"ok","result":true,"content":body[bytes]}]
        - iter[{"code":2, "msg":"iter data","result":true}]
        - iter[{"code":0, "msg":"error","result":false}]