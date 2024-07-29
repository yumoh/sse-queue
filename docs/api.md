

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
        - {"code":0, "msg":"ok","ok":true, "data":true}
        - {"code":1, "msg":"error","ok":false}
2. get
    - url
        - "/msg/queue/get"
        - "/msg/{queue}/get"
    - method: GET
    - params:
        - queue: string, required
        - timeout: int, optional, default None
    - response json
        - {"code":1,"msg":"error","ok":false}
        - {"code":0,"msg":"ok", "ok":true,"data":optional[string]}
3. pick
    - url
        - "/msg/queue/pick"
        - "/msg/{queue}/pick"
    - method: GET
    - params:
        - queue: string, required
    - response json
        - {"code":1,"msg":"error", "ok":false}
        - {"code":0,"msg":"ok", "ok":true,"data":optional[string]}
4. listen
    - url
        - "/msg/queue/listen"
        - "/msg/{queue}/listen"
    - method: GET
    - params:
        - queue: string, required
    - response stream(sse)
        - {"code":1,"msg":"error", "ok":false}
        - iter[{"code":0,"msg":"ok", "ok":true,"data":string}]

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
        - {"code":0,"msg":"ok", "ok":true}
        - {"code":1,"msg":"error", "ok":false}
2. get
    - url
        - "/storage/get"
        - "/storage/get/{bucket}/{name}"
    - method: GET
    - params:
        - bucket: string, required
        - name: string, required
    - response status: 200 body: any bytes

3. create
    - url
        - "/storage/new"
        - "/storage/new/{bucket}"
    - method: GET
    - params:
        - bucket: string, required
    - response json
        - {"code":0,"msg":"ok", "ok":true}
        - {"code":1,"msg":"error", "ok":false}
        
4. delete
    - url
        - "/storage/del"
        - "/storage/del/{bucket}"
        - "/storage/del/{bucket}/{name}"
    - method: GET
    - params:
        - bucket: string, required
        - name: string, optional
    - response json
        - {"code":0,"msg":"ok", "ok":true}
        - {"code":1,"msg":"error", "ok":false}


5. exists
    - url
        - "/storage/exists"
        - "/storage/exists/{bucket}/{name}"
    - method: GET
    - params:
        - bucket: string, required
        - name: string, required
    - response json
        - {"code":0,"msg":"ok", "ok":true}
        - {"code":1,"msg":"error", "ok":false}

6. append
    - url
        - "/storage/append"
        - "/storage/append/{bucket}/{name}"
    - method: POST
    - params:
        - bucket: string, required
        - name: string, required
    - body: bytes
    - response json
        - {"code":0,"msg":"ok", "ok":true}
        - {"code":1,"msg":"error", "ok":false}

7. closeappend
    - url
        - "/storage/closeappend"
        - "/storage/closeappend/{bucket}/{name}"
    - method: GET
    - params:
        - bucket: string, required
        - name: string, required
    - response json
        - {"code":0,"msg":"ok", "ok":true}
        - {"code":1,"msg":"error", "ok":false}

8. fsize
    - url
        - "/storage/fsize"
        - "/storage/fsize/{bucket}/{name}"
    - method: GET
    - params:
        - bucket: string, required
        - name: string, required
    - response json
        - {"code":0,"msg":"ok", "ok":true,"data":123456}
        - {"code":1,"msg":"error", "ok":false}
        
9. list bucket objects
    - url
      - "/storage/list"
      - "/storage/list/{bucket}"
    - method: GET
    - params:
       - filter: string, optional
    - response json
      - {"code":0,"msg":"ok", "ok":true,"data":["a.txt","b.txt"]}
      - {"code":1,"msg":"error", "ok":false}
  
## online log
1. post log stream
    - url
        - "/onlinelog/upload/<channel>/<name>"
        - "/onlinelog/upload?channel=<channel>&name=<name>"
    - method: POST
        - body: bytes
    - response string
        - "ok"

