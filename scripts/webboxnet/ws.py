import base64
import hashlib
import os
import socket
import ssl
import struct
import urllib.parse

GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
OP_BINARY = 0x2
OP_CLOSE = 0x8
OP_PING = 0x9
OP_PONG = 0xA


class WebSocketClient:
    def __init__(self, url):
        self.url = urllib.parse.urlparse(url)
        self.sock = None

    def connect(self):
        if self.url.scheme not in ("ws", "wss"):
            raise ValueError("hub URL must use ws:// or wss://")
        port = self.url.port or (443 if self.url.scheme == "wss" else 80)
        raw = socket.create_connection((self.url.hostname, port), timeout=10)
        if self.url.scheme == "wss":
            self.sock = ssl.create_default_context().wrap_socket(raw, server_hostname=self.url.hostname)
        else:
            self.sock = raw
        self._handshake()

    def fileno(self):
        return self.sock.fileno()

    def close(self):
        if self.sock:
            self.sock.close()
            self.sock = None

    def send_binary(self, payload):
        self.sock.sendall(encode_frame(OP_BINARY, payload, masked=True))

    def recv_binary(self):
        while True:
            frame = read_frame(self.sock)
            if frame is None:
                return None
            opcode, payload = frame
            if opcode == OP_BINARY:
                return payload
            if opcode == OP_PING:
                self.sock.sendall(encode_frame(OP_PONG, payload, masked=True))
            if opcode == OP_CLOSE:
                return None

    def _handshake(self):
        key = base64.b64encode(os.urandom(16)).decode("ascii")
        path = self.url.path or "/"
        if self.url.query:
            path += "?" + self.url.query
        host = self.url.hostname
        if self.url.port:
            host += f":{self.url.port}"
        req = (
            f"GET {path} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\n"
            f"Connection: Upgrade\r\nSec-WebSocket-Key: {key}\r\n"
            "Sec-WebSocket-Version: 13\r\n\r\n"
        )
        self.sock.sendall(req.encode("ascii"))
        response = self.sock.recv(4096).decode("iso-8859-1")
        if " 101 " not in response.split("\r\n", 1)[0]:
            raise RuntimeError("WebSocket hub rejected upgrade")
        expected = accept_value(key)
        if f"Sec-WebSocket-Accept: {expected}".lower() not in response.lower():
            raise RuntimeError("WebSocket hub returned invalid accept key")


def accept_value(key):
    digest = hashlib.sha1((key + GUID).encode("ascii")).digest()
    return base64.b64encode(digest).decode("ascii")


def encode_frame(opcode, payload, masked=False):
    first = 0x80 | opcode
    length = len(payload)
    header = bytearray([first])
    if length < 126:
        header.append(length | (0x80 if masked else 0))
    elif length <= 0xFFFF:
        header.extend([126 | (0x80 if masked else 0), *struct.pack("!H", length)])
    else:
        header.extend([127 | (0x80 if masked else 0), *struct.pack("!Q", length)])
    if not masked:
        return bytes(header) + payload
    mask = os.urandom(4)
    body = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
    return bytes(header) + mask + body


def read_frame(reader):
    head = reader.recv(2)
    if len(head) < 2:
        return None
    first, second = head
    masked = second & 0x80
    length = second & 0x7F
    if length == 126:
        length = struct.unpack("!H", recv_exact(reader, 2))[0]
    elif length == 127:
        length = struct.unpack("!Q", recv_exact(reader, 8))[0]
    mask = recv_exact(reader, 4) if masked else b""
    payload = recv_exact(reader, length)
    if masked:
        payload = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
    return first & 0x0F, payload


def recv_exact(reader, size):
    chunks = []
    remaining = size
    while remaining:
        chunk = reader.recv(remaining)
        if not chunk:
            raise EOFError("WebSocket connection closed")
        chunks.append(chunk)
        remaining -= len(chunk)
    return b"".join(chunks)
