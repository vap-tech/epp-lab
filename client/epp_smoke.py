#!/usr/bin/env python3
"""EPP TLS/mTLS smoke client for local and VPS integration checks.

Without contact-specific flags this verifies a complete Contact lifecycle over
EPP: create, info, a one-field update, a multi-field update, check, delete and
a final availability check.
"""

import argparse
import os
import socket
import ssl
import struct
import uuid
from dataclasses import dataclass
from xml.etree import ElementTree
from xml.sax.saxutils import escape


EPP_NS = "urn:ietf:params:xml:ns:epp-1.0"
CONTACT_NS = "urn:ietf:params:xml:ns:contact-1.0"
DOMAIN_NS = "urn:ietf:params:xml:ns:domain-1.0"


@dataclass(frozen=True)
class ContactFixture:
    contact_id: str
    auth_info: str
    initial_email: str = "contact@example.test"
    updated_email: str = "updated-contact@example.test"
    updated_voice: str = "+79990000000"
    updated_organization: str = "EPP Lab Smoke Test"
    updated_city: str = "Saint Petersburg"


@dataclass(frozen=True)
class DomainFixture:
    name: str
    auth_info: str
    updated_auth_info: str
    nameserver: str = "ns1.external.example"
    updated_nameserver: str = "ns2.external.example"


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


def contact_fixture(require_existing: bool = False) -> ContactFixture:
    contact_id = os.getenv("EPP_CONTACT_ID")
    if require_existing and not contact_id:
        raise RuntimeError("set EPP_CONTACT_ID when running an individual contact command")
    return ContactFixture(
        contact_id=contact_id or f"C{uuid.uuid4().hex[:10].upper()}",
        auth_info=os.getenv("EPP_CONTACT_AUTHINFO", f"test-{uuid.uuid4().hex}"),
    )


def domain_fixture() -> DomainFixture:
    name = os.getenv("EPP_DOMAIN_NAME", f"smoke-{uuid.uuid4().hex[:10]}.com")
    auth_info = os.getenv("EPP_DOMAIN_AUTHINFO", f"domain-{uuid.uuid4().hex}")
    return DomainFixture(name, auth_info, f"updated-{uuid.uuid4().hex}")


def command_xml(
    command: str,
    cl_trid: str,
    contact: ContactFixture | None = None,
    domain: DomainFixture | None = None,
    client_id: str | None = None,
    password: str | None = None,
) -> bytes:
    if command == "login":
        client_id = escape(client_id or os.environ["EPP_CLIENT_ID"])
        password = escape(password or os.environ["EPP_PASSWORD"])
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><login>
<clID>{client_id}</clID><pw>{password}</pw>
<options><version>1.0</version><lang>en</lang></options>
<svcs><objURI>urn:ietf:params:xml:ns:domain-1.0</objURI>
<objURI>{CONTACT_NS}</objURI></svcs>
</login><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    if command == "logout":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><logout/>
<clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    if command == "hello":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><hello/></command></epp>'''.encode()
    if command.startswith("domain:"):
        if domain is None:
            raise ValueError(f"{command} requires a domain fixture")
        name = escape(domain.name)
        if command == "domain:check":
            return f'''<?xml version="1.0"?><epp xmlns="{EPP_NS}"><command><check>
<domain:check xmlns:domain="{DOMAIN_NS}"><domain:name>{name}</domain:name></domain:check>
</check><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
        if command == "domain:create":
            registrant = os.getenv("EPP_DOMAIN_REGISTRANT", "")
            registrant_xml = f"<domain:registrant>{escape(registrant)}</domain:registrant>" if registrant else ""
            return f'''<?xml version="1.0"?><epp xmlns="{EPP_NS}"><command><create>
<domain:create xmlns:domain="{DOMAIN_NS}"><domain:name>{name}</domain:name>
<domain:ns><domain:hostAttr><domain:hostName>{escape(domain.nameserver)}</domain:hostName></domain:hostAttr></domain:ns>
{registrant_xml}<domain:authInfo><domain:pw>{escape(domain.auth_info)}</domain:pw></domain:authInfo>
</domain:create></create><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
        if command == "domain:info":
            return f'''<?xml version="1.0"?><epp xmlns="{EPP_NS}"><command><info>
<domain:info xmlns:domain="{DOMAIN_NS}"><domain:name>{name}</domain:name>
</domain:info></info><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
        if command == "domain:update":
            return f'''<?xml version="1.0"?><epp xmlns="{EPP_NS}"><command><update>
<domain:update xmlns:domain="{DOMAIN_NS}"><domain:name>{name}</domain:name>
<domain:add><domain:ns><domain:hostAttr><domain:hostName>{escape(domain.updated_nameserver)}</domain:hostName></domain:hostAttr></domain:ns></domain:add>
<domain:rem><domain:ns><domain:hostAttr><domain:hostName>{escape(domain.nameserver)}</domain:hostName></domain:hostAttr></domain:ns></domain:rem>
<domain:chg><domain:authInfo><domain:pw>{escape(domain.updated_auth_info)}</domain:pw></domain:authInfo></domain:chg>
</domain:update></update><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
        if command == "domain:delete":
            return f'''<?xml version="1.0"?><epp xmlns="{EPP_NS}"><command><delete>
<domain:delete xmlns:domain="{DOMAIN_NS}"><domain:name>{name}</domain:name></domain:delete>
</delete><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
        raise ValueError(f"unsupported command: {command}")

    if contact is None:
        raise ValueError(f"{command} requires a contact fixture")

    contact_id = escape(contact.contact_id)
    if command == "contact:create":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><create>
<contact:create xmlns:contact="{CONTACT_NS}">
<contact:id>{contact_id}</contact:id>
<contact:postalInfo type="int"><contact:name>Test Contact</contact:name>
<contact:addr><contact:street>Test Street 1</contact:street><contact:city>Moscow</contact:city>
<contact:cc>RU</contact:cc></contact:addr></contact:postalInfo>
<contact:voice>+70000000000</contact:voice><contact:email>{escape(contact.initial_email)}</contact:email>
<contact:authInfo><contact:pw>{escape(contact.auth_info)}</contact:pw></contact:authInfo>
</contact:create></create><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    if command == "contact:check":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><check>
<contact:check xmlns:contact="{CONTACT_NS}"><contact:id>{contact_id}</contact:id></contact:check>
</check><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    if command == "contact:info":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><info>
<contact:info xmlns:contact="{CONTACT_NS}"><contact:id>{contact_id}</contact:id></contact:info>
</info><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    if command == "contact:update-email":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><update>
<contact:update xmlns:contact="{CONTACT_NS}"><contact:id>{contact_id}</contact:id>
<contact:chg><contact:email>{escape(contact.updated_email)}</contact:email></contact:chg></contact:update>
</update><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    if command == "contact:update-details":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><update>
<contact:update xmlns:contact="{CONTACT_NS}"><contact:id>{contact_id}</contact:id>
<contact:chg><contact:voice>{escape(contact.updated_voice)}</contact:voice>
<contact:postalInfo type="int"><contact:org>{escape(contact.updated_organization)}</contact:org>
<contact:addr><contact:city>{escape(contact.updated_city)}</contact:city></contact:addr>
</contact:postalInfo></contact:chg></contact:update>
</update><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    if command == "contact:delete":
        return f'''<?xml version="1.0"?>
<epp xmlns="{EPP_NS}"><command><delete>
<contact:delete xmlns:contact="{CONTACT_NS}"><contact:id>{contact_id}</contact:id></contact:delete>
</delete><clTRID>{escape(cl_trid)}</clTRID></command></epp>'''.encode()
    raise ValueError(f"unsupported command: {command}")


def response_code(response: str) -> str:
    root = ElementTree.fromstring(response)
    result = root.find(f".//{{{EPP_NS}}}result")
    return result.attrib["code"] if result is not None else "unknown"


def send_command(sock: ssl.SSLSocket, command: str, contact: ContactFixture | None = None, domain: DomainFixture | None = None) -> str:
    trid = f"client-{uuid.uuid4()}"
    sock.sendall(frame(command_xml(command, trid, contact, domain)))
    response = read_frame(sock)
    code = response_code(response)
    print(f"{command}: {code}")
    if code != "1000":
        raise RuntimeError(f"{command} failed with EPP code {code}")
    return response


def require_xml_value(response: str, tag: str, expected: str) -> None:
    value = ElementTree.fromstring(response).findtext(f".//{{{CONTACT_NS}}}{tag}")
    if value != expected:
        raise RuntimeError(f"contact:info returned {tag}={value!r}; expected {expected!r}")


def verify_contact_info(response: str, contact: ContactFixture, updated: bool) -> None:
    require_xml_value(response, "id", contact.contact_id)
    require_xml_value(response, "email", contact.updated_email if updated else contact.initial_email)
    if updated:
        require_xml_value(response, "voice", contact.updated_voice)
        require_xml_value(response, "org", contact.updated_organization)
        require_xml_value(response, "city", contact.updated_city)


def verify_availability(response: str, contact: ContactFixture, available: bool) -> None:
    root = ElementTree.fromstring(response)
    contact_id = root.find(f".//{{{CONTACT_NS}}}id")
    expected = "1" if available else "0"
    if contact_id is None or contact_id.text != contact.contact_id or contact_id.attrib.get("avail") != expected:
        state = "available" if available else "unavailable"
        raise RuntimeError(f"contact:check did not report the contact as {state}")
    print(f"contact:check: {'available' if available else 'unavailable'} as expected")


def run_full_contact_cycle(sock: ssl.SSLSocket) -> None:
    contact = contact_fixture()
    print(f"contact fixture: {contact.contact_id}")
    send_command(sock, "contact:create", contact)
    verify_contact_info(send_command(sock, "contact:info", contact), contact, updated=False)
    send_command(sock, "contact:update-email", contact)
    send_command(sock, "contact:update-details", contact)
    verify_contact_info(send_command(sock, "contact:info", contact), contact, updated=True)
    verify_availability(send_command(sock, "contact:check", contact), contact, available=False)
    send_command(sock, "contact:delete", contact)
    verify_availability(send_command(sock, "contact:check", contact), contact, available=True)


def verify_domain_info(response: str, domain: DomainFixture, updated: bool) -> None:
    root = ElementTree.fromstring(response)
    name = root.findtext(f".//{{{DOMAIN_NS}}}name")
    if name != domain.name:
        raise RuntimeError(f"domain:info returned name={name!r}; expected {domain.name!r}")
    expected_ns = domain.updated_nameserver if updated else domain.nameserver
    hosts = [node.text for node in root.findall(f".//{{{DOMAIN_NS}}}hostName")]
    if expected_ns not in hosts:
        raise RuntimeError(f"domain:info did not return nameserver {expected_ns!r}")


def verify_domain_availability(response: str, domain: DomainFixture, available: bool) -> None:
    root = ElementTree.fromstring(response)
    node = root.find(f".//{{{DOMAIN_NS}}}name")
    expected = "1" if available else "0"
    if node is None or node.text != domain.name or node.attrib.get("avail") != expected:
        raise RuntimeError(f"domain:check availability mismatch for {domain.name}")


def run_full_domain_cycle(sock: ssl.SSLSocket) -> None:
    domain = domain_fixture()
    print(f"domain fixture: {domain.name}")
    verify_domain_availability(send_command(sock, "domain:check", domain=domain), domain, True)
    send_command(sock, "domain:create", domain=domain)
    verify_domain_info(send_command(sock, "domain:info", domain=domain), domain, False)
    send_command(sock, "domain:update", domain=domain)
    verify_domain_info(send_command(sock, "domain:info", domain=domain), domain, True)
    verify_domain_availability(send_command(sock, "domain:check", domain=domain), domain, False)
    send_command(sock, "domain:delete", domain=domain)
    verify_domain_availability(send_command(sock, "domain:check", domain=domain), domain, True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default=os.getenv("EPP_HOST", "epp-lab.space-flow.dev"))
    parser.add_argument("--port", type=int, default=int(os.getenv("EPP_PORT", "700")))
    parser.add_argument("--ca", default=os.getenv("EPP_SERVER_CA"))
    parser.add_argument("--cert", default=os.getenv("EPP_CLIENT_CERT"), required=False)
    parser.add_argument("--key", default=os.getenv("EPP_CLIENT_KEY"), required=False)
    parser.add_argument("--wrong-password", action="store_true")
    parser.add_argument("--wrong-client-id", action="store_true")
    parser.add_argument("--create-contact", action="store_true")
    parser.add_argument("--check-contact", action="store_true")
    parser.add_argument("--info-contact", action="store_true")
    parser.add_argument("--update-contact-email", action="store_true")
    parser.add_argument("--delete-contact", action="store_true")
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
            trid = f"client-{uuid.uuid4()}"
            sock.sendall(frame(command_xml("login", trid, client_id=login_client_id, password=login_password)))
            login_response = read_frame(sock)
            login_code = response_code(login_response)
            print(f"login: {login_code}")
            expected = "2200" if args.wrong_password or args.wrong_client_id else "1000"
            if login_code != expected:
                raise RuntimeError(f"login failed with EPP code {login_code}")
            if expected != "1000":
                return

            trid = f"client-{uuid.uuid4()}"
            sock.sendall(frame(command_xml("hello", trid)))
            hello = read_frame(sock)
            if "<greeting>" not in hello:
                raise RuntimeError("hello did not return a greeting")
            print("hello: greeting ok")

            requested = [
                (args.create_contact, "contact:create"),
                (args.check_contact, "contact:check"),
                (args.info_contact, "contact:info"),
                (args.update_contact_email, "contact:update-email"),
                (args.delete_contact, "contact:delete"),
            ]
            if any(enabled for enabled, _ in requested):
                contact = contact_fixture(require_existing=True)
                for enabled, command in requested:
                    if enabled:
                        send_command(sock, command, contact)
            else:
                run_full_contact_cycle(sock)
                run_full_domain_cycle(sock)

            send_command(sock, "logout")


if __name__ == "__main__":
    main()
