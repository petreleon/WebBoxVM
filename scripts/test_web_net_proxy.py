import io
import unittest

from scripts.web_net_proxy import NetworkProxyHub, encode_frame, read_frame, websocket_accept


class NetworkProxyTests(unittest.TestCase):
    def test_websocket_accept_matches_rfc_example(self):
        accept = websocket_accept("dGhlIHNhbXBsZSBub25jZQ==")

        self.assertEqual(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=")

    def test_binary_frame_round_trip_without_mask(self):
        frame = encode_frame(0x2, b"abcd")

        self.assertEqual(read_frame(io.BytesIO(frame)), (0x2, b"abcd"))

    def test_masked_client_frame_is_unmasked(self):
        payload = b"abcd"
        mask = b"\x01\x02\x03\x04"
        masked = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
        frame = bytes([0x82, 0x80 | len(payload)]) + mask + masked

        self.assertEqual(read_frame(io.BytesIO(frame)), (0x2, payload))

    def test_hub_broadcast_skips_sender(self):
        hub = NetworkProxyHub()
        sender = FakePeer()
        receiver = FakePeer()
        hub.add(sender)
        hub.add(receiver)

        hub.broadcast(sender, b"frame")

        self.assertEqual(sender.frames, [])
        self.assertEqual(receiver.frames, [b"frame"])


class FakePeer:
    def __init__(self):
        self.frames = []

    def send_binary(self, payload):
        self.frames.append(payload)


if __name__ == "__main__":
    unittest.main()
