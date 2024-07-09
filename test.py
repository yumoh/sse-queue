import requests
import toml
import time

def load_server():
    config = toml.load("data/config.toml")
    https_on = "ssl" in config
    bind = config['server']['bind']
    prefix = "https" if https_on else "http"
    return f'{prefix}://{bind}'

server = load_server()
print(f"server: {server}")

queue="put-get"
data="msg1"

def test_ping():
    result=requests.get(f"{server}/ping",timeout=2).text
    assert result=='pong' ,"ping failed"

class Msg:
    def __init__(self,queue:str):
        self.queue = queue
    
    def put(self,data:str)->dict:
         result=requests.post(f"{server}/msg/{self.queue}/put",data=data,timeout=5).json()
         return result
    
    def get(self,timeout: int = 0)->dict:
        if timeout == 0:
            result = requests.get(f"{server}/msg/{self.queue}/get",timeout=5).json()
            return result
        else:
            result = requests.get(f"{server}/msg/{self.queue}/get?timeout={timeout}",timeout=timeout+2).json()
            return result
    
    def pick(self,index:int = 0)->dict:
        result = requests.get(f"{server}/msg/{self.queue}/pick/{index}",timeout=5).json()
        return result
    
    def last(self) -> dict:
        result = requests.get(f"{server}/msg/{self.queue}/last",timeout=5).json()
        return result
    
    def first(self) -> dict:
        result = requests.get(f"{server}/msg/{self.queue}/first",timeout=5).json()
        return result

    def listen2(self):
        req = requests.get(f"{server}/msg/{self.queue}/listen",stream=True)
        for line in req.iter_lines():
            if not line:
                time.sleep(0.1)
                continue
            yield line

    def listen(self):
        from requests_sse import EventSource, InvalidStatusCodeError, InvalidContentTypeError
        url = f"{server}/msg/{self.queue}/listen"
        with EventSource(url) as event_source:
            try:
                for event in event_source:
                    if not event.data:
                        time.sleep(0.1)
                        continue
                    yield event.data
            except InvalidStatusCodeError:
                pass
            except InvalidContentTypeError:
                pass
            except requests.RequestException:
                pass
def test_put():
    result=requests.post(f"{server}/msg/{queue}/put",data=data,timeout=5).json()
    assert result['code'],result['msg']

def test_get():
    result = requests.get(f"{server}/msg/{queue}/get",timeout=2).json()
    assert result['code'],result['msg']
    print(result['content'])

def test_put_get():
    msg = Msg(queue)
    for data in ["msg1","msg2","msg3"]:
        result = msg.put(data)
        assert result['code'],result['msg']
    for i in range(3):
        result = msg.get()
        assert result['code'],result['msg']
        print(result['content'])

def test_listen():
    msg = Msg(queue)
    for data in ["msg1","msg2","msg3"]:
        result = msg.put(data)
        assert result['code'],result['msg']
    for line in msg.listen():
        print(line)

def test_pick():
    msg = Msg(queue)
    for data in ["msg1","msg2","msg3"]:
        result = msg.put(data)
        assert result['code'],result['msg']

    for i in range(4):
        result = msg.pick(i)
        print(result)

def test_last_first():
    msg = Msg(queue)
    for data in ["msg1","msg2","msg3"]:
        result = msg.put(data)
        assert result['code'],result['msg']
    last = msg.last()
    print("last:",last)
    first = msg.first()
    print("first:",first)

def release():
    test_ping()
    test_put()
    test_get()
    test_put_get()
    test_listen()
    test_pick()
    test_last_first()

if __name__ == '__main__':
    test_last_first()