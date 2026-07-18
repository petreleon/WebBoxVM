#!/usr/bin/env python3
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

from web_net_proxy import NetworkProxyHub, try_handle_network_websocket


class NoCacheHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-store, max-age=0")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Resource-Policy", "same-origin")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")
        super().end_headers()


class WebBoxHandler(NoCacheHandler):
    net_hub = NetworkProxyHub()

    def do_GET(self):
        if try_handle_network_websocket(self, self.net_hub):
            return
        super().do_GET()


def main():
    parser = argparse.ArgumentParser(description="Serve WebBoxVM without browser cache.")
    parser.add_argument("--directory", default="web")
    parser.add_argument("--host", default="")
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()
    handler = partial(WebBoxHandler, directory=args.directory)
    with ThreadingHTTPServer((args.host, args.port), handler) as server:
        print(f"Serving {args.directory} at http://localhost:{args.port}/")
        print(f"Network proxy at ws://localhost:{args.port}/webboxvm-net")
        server.serve_forever()


if __name__ == "__main__":
    main()
