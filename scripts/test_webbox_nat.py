import struct
import unittest
from unittest.mock import mock_open, patch

from scripts.webboxnet.packets import (
    BROADCAST_IP,
    BROADCAST_MAC,
    DHCP_MAGIC,
    DHCP_CLIENT,
    DHCP_SERVER,
    ETH_ARP,
    ETH_IPV4,
    NatConfig,
    dhcp_message_type,
    ethernet,
    host_reply,
    ipv4_packet,
    ipv4_payload,
    udp_packet,
)
from scripts.webboxnet import host
from scripts.webboxnet.ws import accept_value, encode_frame, read_frame

GUEST_MAC = bytes.fromhex("025742564d01")
GATEWAY_MAC = bytes.fromhex("025742564d02")


class NatPacketTests(unittest.TestCase):
    def setUp(self):
        self.config = NatConfig("10.0.2.2", "10.0.2.15", "1.1.1.1", "02:57:42:56:4d:02")

    def test_arp_request_for_gateway_gets_reply(self):
        payload = struct.pack("!HHBBH", 1, ETH_IPV4, 6, 4, 1)
        payload += GUEST_MAC + ip("10.0.2.15") + b"\0" * 6 + ip("10.0.2.2")
        reply = host_reply(ethernet(BROADCAST_MAC, GUEST_MAC, ETH_ARP, payload), self.config)

        self.assertEqual(reply[:6], GUEST_MAC)
        self.assertEqual(reply[6:12], GATEWAY_MAC)
        self.assertEqual(struct.unpack("!H", reply[20:22])[0], 2)

    def test_dhcp_discover_gets_offer(self):
        reply = host_reply(dhcp_frame(1), self.config)

        bootp = bootp_reply(reply)
        self.assertEqual(bootp[0], 2)
        self.assertEqual(bootp[16:20], ip("10.0.2.15"))
        self.assertEqual(dhcp_message_type(bootp), 2)

    def test_dhcp_request_gets_ack(self):
        reply = host_reply(dhcp_frame(3), self.config)

        self.assertEqual(dhcp_message_type(bootp_reply(reply)), 5)

    def test_dhcp_ack_advertises_configured_dns_server(self):
        reply = host_reply(dhcp_frame(3), self.config)

        self.assertEqual(dhcp_option(bootp_reply(reply), 6), ip("1.1.1.1"))


class WebSocketClientFrameTests(unittest.TestCase):
    def test_accept_value_matches_rfc_example(self):
        self.assertEqual(accept_value("dGhlIHNhbXBsZSBub25jZQ=="), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=")

    def test_masked_client_frame_decodes(self):
        frame = encode_frame(0x2, b"frame", masked=True)

        self.assertTrue(frame[1] & 0x80)
        self.assertEqual(read_frame(FakeSocket(frame)), (0x2, b"frame"))


class LinuxHostSetupTests(unittest.TestCase):
    def test_configure_sets_tap_gateway_mac(self):
        commands = []
        old_run = host.run
        old_rules = host.add_iptables_rules
        old_which = host.shutil.which
        host.run = lambda cmd: commands.append(cmd)
        host.add_iptables_rules = lambda *args: None
        host.shutil.which = lambda _: True
        try:
            host.configure_linux_nat(
                "webbox0",
                "10.0.2.2/24",
                "10.0.2.0/24",
                outbound="eth0",
                gateway_mac="aa:bb",
            )
        finally:
            host.run = old_run
            host.add_iptables_rules = old_rules
            host.shutil.which = old_which

        self.assertIn(["ip", "link", "set", "dev", "webbox0", "address", "aa:bb"], commands)

    def test_ip_forwarding_falls_back_to_proc_file(self):
        old_which = host.shutil.which
        host.shutil.which = lambda _: None
        try:
            with patch("builtins.open", mock_open()) as opened, patch("builtins.print"):
                host.enable_ipv4_forwarding()
        finally:
            host.shutil.which = old_which

        opened.assert_called_once_with("/proc/sys/net/ipv4/ip_forward", "w", encoding="ascii")
        opened().write.assert_called_once_with("1\n")


class FakeSocket:
    def __init__(self, data):
        self.data = bytearray(data)

    def recv(self, size):
        chunk = self.data[:size]
        del self.data[:size]
        return bytes(chunk)


def dhcp_frame(message_type):
    bootp = bytearray(240)
    bootp[0:4] = b"\x01\x01\x06\x00"
    bootp[4:8] = b"\x12\x34\x56\x78"
    bootp[10:12] = b"\x80\x00"
    bootp[28:34] = GUEST_MAC
    bootp[236:240] = DHCP_MAGIC
    bootp += bytes([53, 1, message_type, 255])
    udp = udp_packet(DHCP_CLIENT, DHCP_SERVER, bytes(bootp))
    ipv4 = ipv4_packet(b"\0\0\0\0", BROADCAST_IP, 17, udp)
    return ethernet(BROADCAST_MAC, GUEST_MAC, ETH_IPV4, ipv4)


def bootp_reply(frame):
    parsed = ipv4_payload(frame)
    self_udp = parsed[3]
    length = struct.unpack("!H", self_udp[4:6])[0]
    return self_udp[8:length]


def dhcp_option(bootp, target):
    index = 240
    while index < len(bootp):
        code = bootp[index]
        index += 1
        if code == 255:
            return None
        size = bootp[index]
        index += 1
        value = bootp[index : index + size]
        index += size
        if code == target:
            return value
    return None


def ip(addr):
    return bytes(map(int, addr.split(".")))


if __name__ == "__main__":
    unittest.main()
