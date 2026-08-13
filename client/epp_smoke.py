#!/usr/bin/env python3
"""Minimal EPP TLS smoke client for local/VPS integration checks."""

import argparse
import os
import socket
import ssl
import struct
import uuid
from xml.etree import ElementTree


def frame(payload: bytes) -> bytes:
    return struct.pack(">I", len(payload) + 4) + payload


def read_frame(sock: ssl.SSLSocket) -> str:
    header = sock.recv(4)
    if len(header) != 4:
        raise RuntimeError("unexpected EOF while reading EPP frame header")
    length = struct.unpack(">I", header)[0]
    payload = bytearray()
    while len(payload) < length - 4:
        chunk = sock.recv(length - 4 - len(payload))
        if not chunk:
            raise RuntimeError("unexpected EOF while reading EPP frame body")
        payload.extend(chunk)
    return payload.decode()


def command_xml(command: str, cl_trid: str, client_id: str | None = None, password: str | None = None) -> bytes:
    if command == "login":
        client_id = client_id or os.environ["EPP_CLIENT_ID"]
        password = password or os.environ["EPP_PASSWORD"]
        return f'''<?xml version="1.0"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><login>
<clID>{client_id}</clID><pw>{password}</pw>
<options><version>1.0</version><lang>en</lang></options>
<svcs><objURI>urn:ietf:params:xml:ns:domain-1.0</objURI></svcs>
</login><clTRID>{cl_trid}</clTRID></command></epp>'''.encode()
    if command == "logout":
        return f'''<?xml version="1.0"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><logout/>
<clTRID>{cl_trid}</clTRID></command></epp>'''.encode()
    if command == "hello":
        return b'''<?xml version="1.0"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0"><command><hello/></command></epp>'''
    raise ValueError(f"unsupported command: {command}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default=os.getenv("EPP_HOST", "epp-lab.space-flow.dev"))
    parser.add_argument("--port", type=int, default=int(os.getenv("EPP_PORT", "700")))
    parser.add_argument("--ca", default=os.getenv("EPP_SERVER_CA"))
    parser.add_argument("--cert", default=os.getenv("EPP_CLIENT_CERT"), required=False)
    parser.add_argument("--key", default=os.getenv("EPP_CLIENT_KEY"), required=False)
    parser.add_argument("--wrong-password", action="store_true")
    parser.add_argument("--wrong-client-id", action="store_true")
    args = parser.parse_args()
    if not args.ca or not args.cert or not args.key:
        parser.error("provide --ca, --cert and --key or corresponding environment variables")

    context = ssl.create_default_context(cafile=args.ca)
    context.load_cert_chain(args.cert, args.key)
    with socket.create_connection((args.host, args.port), timeout=10) as raw:
        with context.wrap_socket(raw, server_hostname=args.host) as sock:
            greeting = read_frame(sock)
            if "<greeting>" not in greeting:
                raise RuntimeError("server greeting was not received")
            print("greeting: ok")

            login_client_id = "UNKNOWN-CLIENT" if args.wrong_client_id else None
            login_password = "wrong-password" if args.wrong_password else None
            for command in ("login", "hello", "logout"):
                trid = f"client-{uuid.uuid4()}"
                sock.sendall(frame(command_xml(command, trid, login_client_id, login_password)))
                response = read_frame(sock)
                if command == "hello":
                    if "<greeting>" not in response:
                        raise RuntimeError("hello did not return a greeting")
                    print("hello: greeting ok")
                    continue
                root = ElementTree.fromstring(response)
                result = root.find(".//{urn:ietf:params:xml:ns:epp-1.0}result")
                code = result.attrib["code"] if result is not None else "unknown"
                print(f"{command}: {code}")
                expected = "2200" if command == "login" and (args.wrong_password or args.wrong_client_id) else "1000"
                if code != expected:
                    raise RuntimeError(f"{command} failed with EPP code {code}")
                if command == "login" and expected != "1000":
                    return


if __name__ == "__main__":
    main()
