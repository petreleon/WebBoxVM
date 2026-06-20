import fcntl
import os
import struct

TUNSETIFF = 0x400454CA
IFF_TAP = 0x0002
IFF_NO_PI = 0x1000


class TapDevice:
    def __init__(self, name):
        self.requested_name = name
        self.name = name
        self.fd = None

    def open(self):
        self.fd = os.open("/dev/net/tun", os.O_RDWR | os.O_NONBLOCK)
        ifr = struct.pack("16sH", self.requested_name.encode("ascii"), IFF_TAP | IFF_NO_PI)
        result = fcntl.ioctl(self.fd, TUNSETIFF, ifr)
        self.name = result[:16].split(b"\0", 1)[0].decode("ascii")
        return self

    def fileno(self):
        return self.fd

    def read_frame(self):
        try:
            return os.read(self.fd, 65535)
        except BlockingIOError:
            return None

    def write_frame(self, frame):
        os.write(self.fd, frame)

    def close(self):
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None
