import shutil
import subprocess


def configure_linux_nat(
    tap,
    gateway_cidr,
    subnet,
    outbound=None,
    gateway_mac=None,
    guest_ip=None,
    guest_mac=None,
):
    run(["ip", "addr", "replace", gateway_cidr, "dev", tap])
    if gateway_mac:
        run(["ip", "link", "set", "dev", tap, "address", gateway_mac])
    run(["ip", "link", "set", tap, "up"])
    if guest_ip and guest_mac:
        add_static_guest_neighbor(tap, guest_ip, guest_mac)
    disable_tap_offloads(tap)
    enable_ipv4_forwarding()
    if shutil.which("iptables"):
        add_iptables_rules(tap, subnet, outbound or default_route_dev())
    else:
        print("iptables not found; configure NAT manually for", subnet)


def add_static_guest_neighbor(tap, guest_ip, guest_mac):
    run(["ip", "neigh", "replace", guest_ip, "lladdr", guest_mac, "dev", tap, "nud", "permanent"])


def disable_tap_offloads(tap):
    if not shutil.which("ethtool"):
        return
    subprocess.run(["ethtool", "-K", tap, "tx", "off", "tso", "off", "gso", "off", "gro", "off"])


def add_iptables_rules(tap, subnet, outbound):
    postrouting = ["-s", subnet]
    if outbound:
        postrouting += ["-o", outbound]
    ensure_iptables(["-t", "nat", "-A", "POSTROUTING", *postrouting, "-j", "MASQUERADE"])
    ensure_iptables(["-A", "FORWARD", "-i", tap, "-j", "ACCEPT"])
    ensure_iptables(
        [
            "-A",
            "FORWARD",
            "-o",
            tap,
            "-m",
            "state",
            "--state",
            "RELATED,ESTABLISHED",
            "-j",
            "ACCEPT",
        ]
    )


def ensure_iptables(args):
    check = ["iptables"] + [("-C" if arg == "-A" else arg) for arg in args]
    result = subprocess.run(check, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if result.returncode != 0:
        run(["iptables", *args])


def default_route_dev():
    result = subprocess.run(["ip", "route", "show", "default"], text=True, capture_output=True)
    words = result.stdout.split()
    return words[words.index("dev") + 1] if "dev" in words else None


def enable_ipv4_forwarding():
    if shutil.which("sysctl"):
        run(["sysctl", "-w", "net.ipv4.ip_forward=1"])
        return
    print("+ write /proc/sys/net/ipv4/ip_forward")
    with open("/proc/sys/net/ipv4/ip_forward", "w", encoding="ascii") as handle:
        handle.write("1\n")


def run(cmd):
    print("+", " ".join(cmd))
    subprocess.run(cmd, check=True)
