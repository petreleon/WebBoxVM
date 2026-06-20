import ipaddress
import struct

ETH_ARP = 0x0806
ETH_IPV4 = 0x0800
UDP_PROTO = 17
DHCP_CLIENT = 68
DHCP_SERVER = 67
DHCP_MAGIC = b"\x63\x82\x53\x63"
BROADCAST_MAC = b"\xff" * 6
BROADCAST_IP = b"\xff" * 4


class NatConfig:
    def __init__(self, gateway_ip, guest_ip, dns_ip, gateway_mac, broadcast_ip="10.0.2.255"):
        self.gateway_ip = ipaddress.IPv4Address(gateway_ip).packed
        self.guest_ip = ipaddress.IPv4Address(guest_ip).packed
        self.dns_ip = ipaddress.IPv4Address(dns_ip).packed
        self.gateway_mac = bytes.fromhex(gateway_mac.replace(":", ""))
        self.broadcast_ip = ipaddress.IPv4Address(broadcast_ip).packed


def host_reply(frame, config):
    if len(frame) < 14:
        return None
    eth_type = struct.unpack("!H", frame[12:14])[0]
    if eth_type == ETH_ARP:
        return arp_reply(frame, config)
    if eth_type == ETH_IPV4:
        return dhcp_reply(frame, config)
    return None


def arp_reply(frame, config):
    payload = frame[14:]
    if len(payload) < 28:
        return None
    htype, ptype, hlen, plen, oper = struct.unpack("!HHBBH", payload[:8])
    if (htype, ptype, hlen, plen, oper) != (1, ETH_IPV4, 6, 4, 1):
        return None
    sender_mac = payload[8:14]
    sender_ip = payload[14:18]
    target_ip = payload[24:28]
    if target_ip != config.gateway_ip:
        return None
    arp = struct.pack("!HHBBH", 1, ETH_IPV4, 6, 4, 2)
    arp += config.gateway_mac + config.gateway_ip + sender_mac + sender_ip
    return ethernet(sender_mac, config.gateway_mac, ETH_ARP, arp)


def dhcp_reply(frame, config):
    ip = ipv4_payload(frame)
    if not ip:
        return None
    src_ip, dst_ip, protocol, payload = ip
    if protocol != UDP_PROTO or len(payload) < 8:
        return None
    src_port, dst_port, length, _ = struct.unpack("!HHHH", payload[:8])
    if (src_port, dst_port) != (DHCP_CLIENT, DHCP_SERVER):
        return None
    bootp = payload[8:length]
    msg_type = dhcp_message_type(bootp)
    if msg_type not in (1, 3):
        return None
    reply_type = 2 if msg_type == 1 else 5
    client_mac = bootp[28:34]
    xid = bootp[4:8]
    bootp_reply = build_bootp(bootp, xid, client_mac, config, reply_type)
    udp = udp_packet(DHCP_SERVER, DHCP_CLIENT, bootp_reply)
    ip_reply = ipv4_packet(config.gateway_ip, BROADCAST_IP, UDP_PROTO, udp)
    return ethernet(BROADCAST_MAC, config.gateway_mac, ETH_IPV4, ip_reply)


def ipv4_payload(frame):
    if len(frame) < 34:
        return None
    ip = frame[14:]
    version = ip[0] >> 4
    ihl = (ip[0] & 0x0F) * 4
    if version != 4 or ihl < 20 or len(ip) < ihl:
        return None
    total = struct.unpack("!H", ip[2:4])[0]
    if total < ihl or len(ip) < total:
        return None
    return ip[12:16], ip[16:20], ip[9], ip[ihl:total]


def dhcp_message_type(bootp):
    if len(bootp) < 240 or bootp[236:240] != DHCP_MAGIC:
        return None
    index = 240
    while index < len(bootp):
        code = bootp[index]
        index += 1
        if code == 255:
            return None
        if code == 0:
            continue
        if index >= len(bootp):
            return None
        size = bootp[index]
        index += 1
        value = bootp[index : index + size]
        index += size
        if code == 53 and value:
            return value[0]
    return None


def build_bootp(request, xid, client_mac, config, reply_type):
    flags = request[10:12] if len(request) >= 12 else b"\x80\x00"
    packet = bytearray(236)
    packet[0:4] = b"\x02\x01\x06\x00"
    packet[4:8] = xid
    packet[10:12] = flags
    packet[16:20] = config.guest_ip
    packet[20:24] = config.gateway_ip
    packet[28:34] = client_mac
    options = [
        opt(53, bytes([reply_type])),
        opt(54, config.gateway_ip),
        opt(51, struct.pack("!I", 3600)),
        opt(1, b"\xff\xff\xff\x00"),
        opt(3, config.gateway_ip),
        opt(6, config.dns_ip),
        opt(28, config.broadcast_ip),
        opt(58, struct.pack("!I", 1800)),
        opt(59, struct.pack("!I", 3150)),
        b"\xff",
    ]
    return bytes(packet) + DHCP_MAGIC + b"".join(options)


def opt(code, value):
    return bytes([code, len(value)]) + value


def ethernet(dst, src, eth_type, payload):
    return dst + src + struct.pack("!H", eth_type) + payload


def udp_packet(src_port, dst_port, payload):
    length = 8 + len(payload)
    return struct.pack("!HHHH", src_port, dst_port, length, 0) + payload


def ipv4_packet(src_ip, dst_ip, proto, payload):
    total = 20 + len(payload)
    header = bytearray(20)
    header[0] = 0x45
    struct.pack_into("!H", header, 2, total)
    header[8] = 64
    header[9] = proto
    header[12:16] = src_ip
    header[16:20] = dst_ip
    struct.pack_into("!H", header, 10, checksum(header))
    return bytes(header) + payload


def checksum(data):
    if len(data) % 2:
        data += b"\x00"
    total = sum(struct.unpack(f"!{len(data) // 2}H", data))
    while total >> 16:
        total = (total & 0xFFFF) + (total >> 16)
    return (~total) & 0xFFFF
