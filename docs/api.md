

## base api
1. index
    - url: "/"
    - method: GET
    - response text 200: "sse event queue"

2. ping
    - url: "/ping"
    - method: GET
    - response text 200: "pong"

3. version
    - url: "/version"
    - method: GET
    - response text "x.x.x"

4. time
    - url: "/time"
    - method: GET
    - response text "yyyy-MM-dd HH:mm:ss"

5. ip
    - url: "/ip"
    - method: GET
    - response text "127.0.0.1"


## msg api
1. put 
    - url
        - "/msg/queue/put"
        - "/msg/{queue}/put"
    - method: POST
    - params:
        - queue: string, required
    - request body: any bytes
    - response json
        - {"code":1, "msg":"ok","result":true}
        - {"code":0, "msg":"error","result":false}
2. get
    - url
        - "/msg/queue/get"
        - "/msg/{queue}/get"
    - method: GET
    - params:
        - queue: string, required
        - timeout: int, optional, default None
    - response json
        - {"code":0,"msg":"error","result":false}
        - {"code":1,"msg":"ok","result":true,"content":bytes}
3. pick
    - url
        - "/msg/queue/pick"
        - "/msg/{queue}/pick"
    - method: GET
    - params:
        - queue: string, required
    - response json
        - {"code":0,"msg":"error","result":false}
        - {"code":1,"msg":"ok","result":true,"content":bytes}
4. listen
    - url
        - "/msg/queue/listen"
        - "/msg/{queue}/listen"
    - method: GET
    - params:
        - queue: string, required
    - response stream(sse)
        - {"code":0,"msg":"error","result":false}
        - iter[{"code":1,"msg":"ok","result":true,"content":bytes}]
        - iter[{"code":2,"msg":"ok","result":false,"content":null}]

## storage api
1. put
    - url
        - "/storage/put"
        - "/storage/put/{bucket}/{name}"
    - method: POST
    - params:
        - bucket: string, required
        - name: string, required
    - request body: any bytes
    - response json
        - {"code":1,"msg":"ok","result":true}
        - {"code":0,"msg":"error","result":false}
2. get
    - url
        - "/storage/get"
        - "/storage/get/{bucket}/{name}"
    - method: GET
    - params:
        - bucket: string, required
        - name: string, required
    - response status: 200 body: any bytes

3. delete
    - url
        - "/storage/del"
        - "/storage/del/{bucket}"
        - "/storage/del/{bucket}/{name}"
    - method: GET
    - params:
        - bucket: string, required
        - name: string, optional
    - response json
        - {"code":1,"msg":"ok","result":true}
        - {"code":0,"msg":"error","result":false}


