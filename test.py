import requests
import toml
import time
import io


def load_server():
    config = toml.load("data/config.toml")
    https_on = "ssl" in config
    bind = config["server"]["bind"]
    prefix = "https" if https_on else "http"
    return f"{prefix}://{bind}"


server = load_server()
print(f"server: {server}")

queue = "put-get"
data = "msg1"


def test_ping():
    print("== ping ==")
    result = requests.get(f"{server}/ping", timeout=2).text
    assert result == "pong", "ping failed"


class Msg:
    def __init__(self, queue: str):
        self.queue = queue

    def put(self, data: str) -> dict:
        result = requests.post(
            f"{server}/msg/{self.queue}/put", data=data, timeout=5
        ).json()
        return result

    def get(self, timeout: int = 0) -> dict:
        if timeout == 0:
            result = requests.get(f"{server}/msg/{self.queue}/get", timeout=5).json()
            return result
        else:
            result = requests.get(
                f"{server}/msg/{self.queue}/get?timeout={timeout}", timeout=timeout + 2
            ).json()
            return result

    def pick(self, index: int = 0) -> dict:
        result = requests.get(
            f"{server}/msg/{self.queue}/pick/{index}", timeout=5
        ).json()
        return result

    def last(self) -> dict:
        result = requests.get(f"{server}/msg/{self.queue}/last", timeout=5).json()
        return result

    def first(self) -> dict:
        result = requests.get(f"{server}/msg/{self.queue}/first", timeout=5).json()
        return result

    def listen2(self):
        req = requests.get(f"{server}/msg/{self.queue}/listen", stream=True)
        for line in req.iter_lines():
            if not line:
                time.sleep(0.1)
                continue
            yield line

    def listen(self,timeout:int = 0):
        from requests_sse import (
            EventSource,
            InvalidStatusCodeError,
            InvalidContentTypeError,
        )
        if timeout > 0:
            url = f"{server}/msg/{self.queue}/listen?timeout={timeout}"
        else:
            url = f"{server}/msg/{self.queue}/listen"
            timeout = None
        with EventSource(url,timeout=timeout) as event_source:
            try:
                for event in event_source:
                    print(event)
                    if event.data == "bye":
                        break
                    yield event.data
            except InvalidStatusCodeError:
                print("InvalidStatusCodeError")
                pass
            except InvalidContentTypeError:
                print("InvalidContentTypeError")
                pass
            except requests.RequestException:
                print("requests.RequestException")
                pass


class Storage:
    def put(self, bucket: str, name: str, data: bytes) -> dict:
        result = requests.post(
            f"{server}/storage/put/{bucket}/{name}", data=data
        ).json()
        return result

    def append(self, bucket: str, name: str, data: bytes) -> dict:
        result = requests.post(
            f"{server}/storage/append/{bucket}/{name}", data=data
        ).json()
        return result

    def close_append(self, bucket: str, name: str) -> dict:
        result = requests.get(
            f"{server}/storage/closeappend/{bucket}/{name}"
        ).json()
        return result

    def put_file(self, bucket: str, name: str, file_path: str) -> dict:
        with open(file_path, "rb") as f:
            data = f.read()
            result = self.put(bucket, name, data)
        return result

    def get(self, bucket: str, name: str) -> bytes:
        result = requests.get(f"{server}/storage/get/{bucket}/{name}")
        assert result.status_code == 200
        return result.content

    def get_file(self, bucket: str, name: str, file_path: str) -> None:
        data = self.get(bucket, name)
        with open(file_path, "wb") as f:
            f.write(data)

    def remove_file(self, bucket: str, name: str,*,exists_ok=False):
        params = {"exists_ok":"true" if exists_ok else "false"}
        result = requests.get(f"{server}/storage/del/{bucket}/{name}",params=params).json()
        assert result["ok"], result["msg"]

    def get_io(self, bucket: str, name: str) -> io.BufferedReader:
        res = requests.get(f"{server}/storage/get/{bucket}/{name}", stream=True)
        assert res.status_code == 200, f"get {bucket}/{name} failed"
        return res.raw


def test_put():
    print("== put ==")
    result = requests.post(f"{server}/msg/{queue}/put", data=data, timeout=5).json()
    assert result["ok"], result["msg"]


def test_get():
    print("== get ==")
    result = requests.get(f"{server}/msg/{queue}/get", timeout=2).json()
    assert result["ok"], result["msg"]
    print(result["data"])


def test_put_get():
    print("== put get ==")
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["ok"], result["msg"]
    for i in range(3):
        result = msg.get()
        assert result["ok"], result["msg"]
        print(result["data"])


def test_listen():
    print("== listen ==")
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["ok"], result["msg"]
    for line in msg.listen(timeout=3):
        print(line)


def test_pick():
    print("== pick ==")
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["ok"], result["msg"]

    for i in range(4):
        result = msg.pick(i)
        print(result)


def test_last_first():
    print("== last first ==")
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["ok"], result["msg"]
    last = msg.last()
    print("last:", last)
    first = msg.first()
    print("first:", first)


def test_upload_file():
    print("== upload file ==")
    s = Storage()
    result = s.put_file("test", "file1", "README.md")
    print(result)
    assert result["ok"], result["msg"]
    s.remove_file("test", "file1")
    s.remove_file("test", "file1",exists_ok=True)

def test_append_file():
    print("== append file ==")
    s = Storage()
    result = s.append("test", "file1-append", b"hello,world1\n")
    assert result["ok"], result["msg"]
    result = s.append("test", "file1-append", b"hello,world2\n")
    assert result["ok"], result["msg"]
    result = s.append("test", "file1-append", b"hello,world3\n")
    assert result["ok"], result["msg"]
    s.close_append("test", "file1-append")
    result = s.append("test", "file1-append", b"\n")
    # s.remove_file("test", "file1-append")
    # s.remove_file("test", "file1-append",exists_ok=True)


def test_download_file():
    print("== download file ==")
    s = Storage()
    result = s.put_file("test", "file1", "README.md")
    result = s.get("test", "file1")
    print(result.decode())


def test_download_stream():
    print("== download stream ==")
    s = Storage()
    stream = s.get_io("test", "file1")
    for line in stream:
        print(line.decode(), end="")


def test_upload_log():
    print("== upload log ==")
    data = [f"lines {i}" for i in range(10)]
    result = requests.post(
        f"{server}/onlinelog/upload/testlog/info.log", data="\n".join(data)
    )
    assert result.status_code == 200, result.text
    print(result.text)


def release():
    test_ping()
    test_put()
    test_get()
    test_put_get()
    test_listen()
    test_pick()
    test_last_first()


def release_storage():
    test_upload_file()
    test_download_file()
    test_download_stream()
    test_append_file()

if __name__ == "__main__":
    release()
    release_storage()
    # test_upload_file()
    # test_append_file()
    # test_download_file()
    # test_download_stream()
    # test_upload_log()
