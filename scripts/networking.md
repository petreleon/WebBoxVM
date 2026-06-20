# Browser NAT Peer

The browser VM sends raw Ethernet frames to `/webboxvm-net`. The NAT peer
connects to that WebSocket hub, creates a Linux TAP interface, answers the
guest's ARP/DHCP bootstrap, and hands all routed IPv4 traffic to Linux.

Run the web app first:

```sh
make web
```

On a Linux host with `CAP_NET_ADMIN`, run:

```sh
sudo python3 scripts/webbox_nat.py --configure-host
```

Defaults:

- Hub: `ws://localhost:8080/webboxvm-net`
- TAP: `webbox0`
- Gateway: `10.0.2.2/24`
- Guest lease: `10.0.2.15`
- DNS lease option: `1.1.1.1`

`--configure-host` assigns the gateway address to the TAP device, enables IPv4
forwarding, and adds iptables forwarding/MASQUERADE rules for the guest subnet.
Without that flag, the service still connects frames but expects you to configure
the host route/NAT rules yourself.

This service is intentionally Linux-only. macOS should run the browser and web
server, while a Linux machine or VM can connect as the NAT peer by passing
`--hub ws://<web-host>:8080/webboxvm-net`.

For local macOS validation through Docker Desktop:

```sh
docker run --rm --privileged \
  -v "$PWD":/work -w /work python:3.14-slim sh -lc '
    apt-get update &&
    apt-get install -y --no-install-recommends iproute2 iptables &&
    python -u scripts/webbox_nat.py \
      --hub ws://host.docker.internal:8080/webboxvm-net \
      --configure-host
  '
```

The container needs `--privileged` so it can create TAP devices, enable IPv4
forwarding, and install iptables NAT rules inside Docker's Linux VM.
