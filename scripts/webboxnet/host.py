import shutil
import subprocess


def configure_linux_nat(tap, gateway_cidr, subnet, outbound=None):
    run(["ip", "addr", "replace", gateway_cidr, "dev", tap])
    run(["ip", "link", "set", tap, "up"])
    run(["sysctl", "-w", "net.ipv4.ip_forward=1"])
    if shutil.which("iptables"):
        add_iptables_rules(tap, subnet, outbound or default_route_dev())
    else:
        print("iptables not found; configure NAT manually for", subnet)


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


def run(cmd):
    print("+", " ".join(cmd))
    subprocess.run(cmd, check=True)
