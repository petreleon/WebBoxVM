import ipaddress
import socket

from .packets import ETH_IPV4, UDP_PROTO, ethernet, ipv4_packet, ipv4_payload, udp_packet

DNS_PORT = 53


class DnsForwarder:
    def __init__(self, upstream=None, timeout=2.0, trace=False):
        self.upstreams = [upstream] if upstream else default_dns_upstreams()
        self.upstream = self.upstreams[0]
        self.timeout = timeout
        self.trace = trace
        self.cache = {}

    def reply(self, frame, config):
        parsed = ipv4_payload(frame)
        if not parsed:
            return None
        src_ip, dst_ip, protocol, payload = parsed
        if protocol != UDP_PROTO or dst_ip != config.dns_ip or len(payload) < 8:
            return None
        src_port, dst_port, length, _ = udp_fields(payload)
        if dst_port != DNS_PORT:
            return None
        self.log(f"query {addr(src_ip)}:{src_port} -> {addr(dst_ip)}")
        answer = self.resolve(payload[8:length])
        if not answer:
            self.log("no answer")
            return None
        self.log(f"reply {len(answer)} bytes")
        udp = udp_packet(DNS_PORT, src_port, answer)
        ip = ipv4_packet(config.dns_ip, src_ip, UDP_PROTO, udp)
        return ethernet(frame[6:12], config.gateway_mac, ETH_IPV4, ip)

    def query(self, payload):
        for upstream in self.upstreams:
            answer = self.query_upstream(payload, upstream)
            if answer:
                return answer
        return None

    def query_upstream(self, payload, upstream):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
                sock.settimeout(self.timeout)
                sock.sendto(payload, (upstream, DNS_PORT))
                return sock.recvfrom(4096)[0]
        except OSError as error:
            print(f"DNS forward failed via {upstream}: {error}")
            return None

    def resolve(self, payload):
        key = payload[2:]
        cached = self.cache.get(key)
        if cached:
            return payload[:2] + cached
        answer = self.query(payload)
        if answer and len(answer) >= 2:
            self.cache[key] = answer[2:]
        return answer

    def log(self, message):
        if getattr(self, "trace", False):
            print(f"DNS {message}", flush=True)


def default_dns_upstream(path="/etc/resolv.conf"):
    return default_dns_upstreams(path)[0]


def default_dns_upstreams(path="/etc/resolv.conf"):
    upstreams = []
    try:
        with open(path, encoding="ascii") as handle:
            for line in handle:
                words = line.split()
                if len(words) >= 2 and words[0] == "nameserver" and is_ipv4(words[1]):
                    upstreams.append(words[1])
    except OSError:
        pass
    for fallback in ("1.1.1.1", "8.8.8.8"):
        if fallback not in upstreams:
            upstreams.append(fallback)
    return upstreams


def udp_fields(payload):
    return (
        int.from_bytes(payload[0:2], "big"),
        int.from_bytes(payload[2:4], "big"),
        int.from_bytes(payload[4:6], "big"),
        int.from_bytes(payload[6:8], "big"),
    )


def is_ipv4(addr):
    try:
        ipaddress.IPv4Address(addr)
        return True
    except ipaddress.AddressValueError:
        return False


def addr(raw):
    return str(ipaddress.IPv4Address(raw))
