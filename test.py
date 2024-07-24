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

    def listen(self):
        from requests_sse import (
            EventSource,
            InvalidStatusCodeError,
            InvalidContentTypeError,
        )

        url = f"{server}/msg/{self.queue}/listen"
        with EventSource(url) as event_source:
            try:
                for event in event_source:
                    # if not event.data:
                    #     time.sleep(0.1)
                    #     continue
                    yield event.data
            except InvalidStatusCodeError:
                pass
            except InvalidContentTypeError:
                pass
            except requests.RequestException:
                pass


class Storage:
    def put(self, bucket: str, name: str, data: bytes) -> dict:
        result = requests.post(
            f"{server}/storage/put/{bucket}/{name}", data=data
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
        assert result["code"], result["msg"]

    def get_io(self, bucket: str, name: str) -> io.BufferedReader:
        res = requests.get(f"{server}/storage/get/{bucket}/{name}", stream=True)
        assert res.status_code == 200, f"get {bucket}/{name} failed"
        return res.raw


def test_put():
    result = requests.post(f"{server}/msg/{queue}/put", data=data, timeout=5).json()
    assert result["code"], result["msg"]


def test_get():
    result = requests.get(f"{server}/msg/{queue}/get", timeout=2).json()
    assert result["code"], result["msg"]
    print(result["content"])


def test_put_get():
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["code"], result["msg"]
    for i in range(3):
        result = msg.get()
        assert result["code"], result["msg"]
        print(result["content"])


def test_listen():
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["code"], result["msg"]
    for line in msg.listen():
        print(line)


def test_pick():
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["code"], result["msg"]

    for i in range(4):
        result = msg.pick(i)
        print(result)


def test_last_first():
    msg = Msg(queue)
    for data in ["msg1", "msg2", "msg3"]:
        result = msg.put(data)
        assert result["code"], result["msg"]
    last = msg.last()
    print("last:", last)
    first = msg.first()
    print("first:", first)


def test_upload_file():
    s = Storage()
    result = s.put_file("test", "file1", "README.md")
    print(result)
    assert result["code"], result["msg"]
    s.remove_file("test", "file1")
    s.remove_file("test", "file1",exists_ok=True)


def test_download_file():
    s = Storage()
    result = s.get("test", "file1")
    print(result.decode())


def test_download_stream():
    s = Storage()
    stream = s.get_io("test", "file1")
    for line in stream:
        print(line.decode(), end="")


def test_upload_log():
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


if __name__ == "__main__":
    test_upload_file()
    # test_download_file()
    # test_download_stream()
    # test_upload_log()
