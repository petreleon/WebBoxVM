import base64
import hashlib
import struct
import threading

GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
MAX_FRAME = 1 << 20
OP_CLOSE = 0x8
OP_PING = 0x9
OP_PONG = 0xA
OP_BINARY = 0x2


class NetworkProxyHub:
    def __init__(self):
        self._lock = threading.Lock()
        self._peers = set()

    def add(self, peer):
        with self._lock:
            self._peers.add(peer)

    def remove(self, peer):
        with self._lock:
            self._peers.discard(peer)

    def broadcast(self, sender, payload):
        with self._lock:
            peers = [peer for peer in self._peers if peer is not sender]
        for peer in peers:
            peer.send_binary(payload)


class WebSocketPeer:
    def __init__(self, writer):
        self._writer = writer
        self._lock = threading.Lock()

    def send_binary(self, payload):
        self._send(OP_BINARY, payload)

    def send_pong(self, payload):
        self._send(OP_PONG, payload)

    def send_close(self):
        self._send(OP_CLOSE, b"")

    def _send(self, opcode, payload):
        with self._lock:
            self._writer.write(encode_frame(opcode, payload))
            self._writer.flush()


def try_handle_network_websocket(handler, hub):
    if handler.path.split("?", 1)[0] != "/webboxvm-net":
        return False
    if handler.headers.get("Upgrade", "").lower() != "websocket":
        handler.send_error(426, "WebSocket upgrade required")
        return True

    key = handler.headers.get("Sec-WebSocket-Key")
    if not key:
        handler.send_error(400, "Missing Sec-WebSocket-Key")
        return True

    accept = websocket_accept(key)
    handler.send_response(101, "Switching Protocols")
    handler.send_header("Upgrade", "websocket")
    handler.send_header("Connection", "Upgrade")
    handler.send_header("Sec-WebSocket-Accept", accept)
    handler.end_headers()
    serve_peer(handler, hub)
    return True


def serve_peer(handler, hub):
    peer = WebSocketPeer(handler.wfile)
    hub.add(peer)
    print(f"Network proxy peer connected from {handler.client_address[0]}")
    try:
        while True:
            frame = read_frame(handler.rfile)
            if frame is None:
                break
            opcode, payload = frame
            if opcode == OP_BINARY:
                hub.broadcast(peer, payload)
            elif opcode == OP_PING:
                peer.send_pong(payload)
            elif opcode == OP_CLOSE:
                peer.send_close()
                break
    finally:
        hub.remove(peer)
        handler.close_connection = True
        print(f"Network proxy peer disconnected from {handler.client_address[0]}")


def websocket_accept(key):
    digest = hashlib.sha1((key + GUID).encode("ascii")).digest()
    return base64.b64encode(digest).decode("ascii")


def read_frame(reader):
    head = reader.read(2)
    if len(head) < 2:
        return None
    first, second = head
    opcode = first & 0x0F
    masked = second & 0x80
    length = second & 0x7F
    if length == 126:
        length = struct.unpack("!H", reader.read(2))[0]
    elif length == 127:
        length = struct.unpack("!Q", reader.read(8))[0]
    if length > MAX_FRAME:
        return None
    mask = reader.read(4) if masked else b""
    payload = reader.read(length)
    if len(payload) < length:
        return None
    if masked:
        payload = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
    return opcode, payload


def encode_frame(opcode, payload):
    first = 0x80 | opcode
    length = len(payload)
    if length < 126:
        header = bytes([first, length])
    elif length <= 0xFFFF:
        header = bytes([first, 126]) + struct.pack("!H", length)
    else:
        header = bytes([first, 127]) + struct.pack("!Q", length)
    return header + payload
