#!/usr/bin/env python3
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class NoCacheHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-store, max-age=0")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")
        super().end_headers()


def main():
    parser = argparse.ArgumentParser(description="Serve WebBoxVM without browser cache.")
    parser.add_argument("--directory", default="web")
    parser.add_argument("--host", default="")
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()
    handler = partial(NoCacheHandler, directory=args.directory)
    with ThreadingHTTPServer((args.host, args.port), handler) as server:
        print(f"Serving {args.directory} at http://localhost:{args.port}/")
        server.serve_forever()


if __name__ == "__main__":
    main()
