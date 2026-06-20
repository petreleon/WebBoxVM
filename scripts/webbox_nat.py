#!/usr/bin/env python3
import argparse
import ipaddress
import select
import signal

from webboxnet.host import configure_linux_nat
from webboxnet.dns import DnsForwarder
from webboxnet.packets import NatConfig, host_reply
from webboxnet.tap import TapDevice
from webboxnet.ws import WebSocketClient

DEFAULT_HUB = "ws://localhost:8080/webboxvm-net"
DEFAULT_GATEWAY_MAC = "02:57:42:56:4d:02"


class NatPeer:
    def __init__(self, args):
        self.args = args
        self.config = NatConfig(args.gateway_ip, args.guest_ip, args.dns_ip, args.gateway_mac)
        self.dns = DnsForwarder(args.dns_upstream, args.dns_timeout)
        self.tap = TapDevice(args.tap)
        self.ws = WebSocketClient(args.hub)
        self.running = True

    def run(self):
        self.tap.open()
        if self.args.configure_host:
            configure_linux_nat(
                self.tap.name,
                self.gateway_cidr(),
                self.subnet(),
                gateway_mac=self.args.gateway_mac,
            )
        else:
            print("Host NAT not configured; pass --configure-host when running as root.")
        self.ws.connect()
        print(f"NAT peer connected: {self.tap.name} <-> {self.args.hub}")
        print(f"DNS proxy: {self.args.dns_ip} -> {self.dns.upstream}")
        while self.running:
            for fd in readable([self.tap, self.ws]):
                if fd is self.tap:
                    self.from_tap()
                else:
                    self.from_ws()

    def stop(self, *_):
        self.running = False

    def close(self):
        self.ws.close()
        self.tap.close()

    def from_tap(self):
        frame = self.tap.read_frame()
        if frame:
            self.ws.send_binary(frame)

    def from_ws(self):
        frame = self.ws.recv_binary()
        if frame is None:
            self.running = False
            return
        reply = host_reply(frame, self.config) or self.dns.reply(frame, self.config)
        if reply:
            self.ws.send_binary(reply)
        else:
            self.tap.write_frame(frame)

    def gateway_cidr(self):
        return f"{self.args.gateway_ip}/{self.args.prefix}"

    def subnet(self):
        network = ipaddress.IPv4Network(f"{self.args.gateway_ip}/{self.args.prefix}", strict=False)
        return str(network)


def readable(devices):
    ready, _, _ = select.select(devices, [], [], 1.0)
    return ready


def parse_args():
    parser = argparse.ArgumentParser(description="Route WebBoxVM browser Ethernet through Linux NAT.")
    parser.add_argument("--hub", default=DEFAULT_HUB)
    parser.add_argument("--tap", default="webbox0")
    parser.add_argument("--gateway-ip", default="10.0.2.2")
    parser.add_argument("--guest-ip", default="10.0.2.15")
    parser.add_argument("--dns-ip", default="1.1.1.1")
    parser.add_argument("--dns-upstream")
    parser.add_argument("--dns-timeout", type=float, default=2.0)
    parser.add_argument("--prefix", type=int, default=24)
    parser.add_argument("--gateway-mac", default=DEFAULT_GATEWAY_MAC)
    parser.add_argument("--configure-host", action="store_true")
    return parser.parse_args()


def main():
    peer = NatPeer(parse_args())
    signal.signal(signal.SIGTERM, peer.stop)
    signal.signal(signal.SIGINT, peer.stop)
    try:
        peer.run()
    finally:
        peer.close()


if __name__ == "__main__":
    main()
