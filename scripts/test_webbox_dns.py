import struct
import unittest
from unittest.mock import mock_open, patch

from scripts.webboxnet.dns import DnsForwarder, default_dns_upstream
from scripts.webboxnet.packets import ETH_IPV4, NatConfig, ethernet, ipv4_packet, ipv4_payload, udp_packet

GUEST_MAC = bytes.fromhex("025742564d01")
GATEWAY_MAC = bytes.fromhex("025742564d02")


class DnsForwarderTests(unittest.TestCase):
    def setUp(self):
        self.config = NatConfig("10.0.2.2", "10.0.2.15", "1.1.1.1", "02:57:42:56:4d:02")

    def test_dns_query_to_advertised_server_gets_host_reply(self):
        query = b"\x12\x34\x01\x00query"
        answer = b"\x12\x34\x81\x80answer"
        forwarder = FakeDnsForwarder(answer)

        reply = forwarder.reply(dns_frame("1.1.1.1", query), self.config)

        src_ip, dst_ip, protocol, udp = ipv4_payload(reply)
        self.assertEqual(forwarder.query_payload, query)
        self.assertEqual(reply[:6], GUEST_MAC)
        self.assertEqual(reply[6:12], GATEWAY_MAC)
        self.assertEqual((src_ip, dst_ip, protocol), (ip("1.1.1.1"), ip("10.0.2.15"), 17))
        self.assertEqual(struct.unpack("!HH", udp[:4]), (53, 40540))
        self.assertEqual(udp[8:], answer)

    def test_dns_forwarder_ignores_other_destinations(self):
        forwarder = FakeDnsForwarder(b"unused")

        self.assertIsNone(forwarder.reply(dns_frame("8.8.8.8", b"query"), self.config))
        self.assertIsNone(forwarder.query_payload)

    def test_dns_forwarder_caches_answers_by_question(self):
        first = b"\x12\x34\x01\x00same-question"
        second = b"\xab\xcd\x01\x00same-question"
        forwarder = FakeDnsForwarder(b"\x12\x34\x81\x80answer")

        first_reply = forwarder.reply(dns_frame("1.1.1.1", first), self.config)
        second_reply = forwarder.reply(dns_frame("1.1.1.1", second), self.config)

        self.assertEqual(forwarder.query_count, 1)
        self.assertEqual(dns_payload(first_reply), b"\x12\x34\x81\x80answer")
        self.assertEqual(dns_payload(second_reply), b"\xab\xcd\x81\x80answer")

    def test_default_dns_upstream_reads_resolv_conf(self):
        resolv = "search local\nnameserver 127.0.0.11\nnameserver 1.1.1.1\n"

        with patch("builtins.open", mock_open(read_data=resolv)):
            self.assertEqual(default_dns_upstream(), "127.0.0.11")

    def test_default_dns_upstream_skips_non_ipv4_nameservers(self):
        resolv = "nameserver fe80::1\nnameserver 9.9.9.9\n"

        with patch("builtins.open", mock_open(read_data=resolv)):
            self.assertEqual(default_dns_upstream(), "9.9.9.9")


def dns_frame(dst_ip, payload):
    udp = udp_packet(40540, 53, payload)
    ipv4 = ipv4_packet(ip("10.0.2.15"), ip(dst_ip), 17, udp)
    return ethernet(GATEWAY_MAC, GUEST_MAC, ETH_IPV4, ipv4)


class FakeDnsForwarder(DnsForwarder):
    def __init__(self, answer):
        self.answer = answer
        self.query_payload = None
        self.query_count = 0
        self.cache = {}

    def query(self, payload):
        self.query_payload = payload
        self.query_count += 1
        return self.answer


def dns_payload(frame):
    _, _, _, udp = ipv4_payload(frame)
    return udp[8:]


def ip(addr):
    return bytes(map(int, addr.split(".")))


if __name__ == "__main__":
    unittest.main()
