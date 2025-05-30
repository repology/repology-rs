#!/usr/bin/env python
# SPDX-FileCopyrightText: Copyright 2025 Dmitry Marakasov <amdmi3@amdmi3.ru>
# SPDX-License-Identifier: GPL-3.0-or-later

import argparse
import asyncio
import errno
import socket
import ssl
import sys
import traceback
from asyncio.exceptions import CancelledError
from dataclasses import dataclass
from typing import Any, List, Optional

import aiohttp
import idna
from aiohttp.abc import AbstractResolver, ResolveResult
from dataclasses_json import dataclass_json


@dataclass_json
@dataclass
class Request:
    request_id: int
    url: str
    use_get: bool
    address: str
    timeout: float


def _full_class_name(cls: Any) -> str:
    if cls.__module__ is None:
        return str(cls.__name__)
    else:
        return str(cls.__module__ + "." + cls.__name__)


def _print_exception_info(e: BaseException, level: int = 0) -> None:
    prefix = "  " * level
    print(
        "{}  Class: {}".format(prefix, _full_class_name(e.__class__)), file=sys.stderr
    )
    print("{}Message: {}".format(prefix, str(e)), file=sys.stderr)
    print(
        "{}  Bases: {}".format(
            prefix, ", ".join((_full_class_name(cls) for cls in e.__class__.mro()))
        ),
        file=sys.stderr,
    )
    errn = getattr(e, "errno", None)
    if errn:
        print("{}  Errno: {}".format(prefix, errn), file=sys.stderr)

    if e.__cause__:
        print("{}Cause:".format(prefix), file=sys.stderr)
        _print_exception_info(e.__cause__, level + 1)


def _classify_exception(e: BaseException) -> Optional[str]:
    if isinstance(e, asyncio.TimeoutError):
        return "Timeout"

    if isinstance(e, aiohttp.client_exceptions.ClientConnectorCertificateError) and e.certificate_error.verify_code == 10:  # type: ignore  # X509_V_ERR_CERT_HAS_EXPIRED
        return "SslCertificateHasExpired"

    if isinstance(e, aiohttp.client_exceptions.ClientConnectorCertificateError) and e.certificate_error.verify_code == 18:  # type: ignore  # X509_V_ERR_DEPTH_ZERO_SELF_SIGNED_CERT
        return "SslCertificateSelfSigned"

    if isinstance(e, aiohttp.client_exceptions.ClientConnectorCertificateError) and e.certificate_error.verify_code == 19:  # type: ignore  # X509_V_ERR_SELF_SIGNED_CERT_IN_CHAIN
        return "SslCertificateSelfSignedInChain"

    if isinstance(e, aiohttp.client_exceptions.ClientConnectorCertificateError) and e.certificate_error.verify_code == 20:  # type: ignore  # X509_V_ERR_UNABLE_TO_GET_ISSUER_CERT_LOCALLY
        return "SslCertificateIncompleteChain"

    if isinstance(e, aiohttp.client_exceptions.ClientConnectorCertificateError) and e.certificate_error.verify_code == 62:  # type: ignore  # X509_V_ERR_HOSTNAME_MISMATCH
        return "SslCertificateHostnameMismatch"

    if isinstance(e, aiohttp.client_exceptions.ClientConnectorCertificateError):
        return "SslError"

    if isinstance(e, aiohttp.client_exceptions.ClientConnectorSSLError):
        return "SslError"

    if isinstance(e, aiohttp.client_exceptions.ServerDisconnectedError):
        return "ServerDisconnected"

    if isinstance(e, ValueError) and str(e) == "URL should be absolute":
        return "InvalidUrl"

    if isinstance(e, ValueError) and str(e) == "Can redirect only to http or https":
        # XXX
        return "InvalidUrl"

    if isinstance(e, aiohttp.client_exceptions.InvalidURL):
        return "InvalidUrl"

    if isinstance(e, UnicodeError):
        return "InvalidUrl"

    if isinstance(e, idna.core.IDNAError):
        return "InvalidUrl"

    if isinstance(e, OSError) and e.errno == errno.ENETUNREACH:
        return "NetworkUnreachable"

    if isinstance(e, OSError) and e.errno == errno.ECONNRESET:
        return "ConnectionResetByPeer"

    if isinstance(e, OSError) and e.errno == errno.ECONNREFUSED:
        return "ConnectionRefused"

    if isinstance(e, OSError) and e.errno == errno.EHOSTUNREACH:
        return "HostUnreachable"

    if isinstance(e, OSError) and e.errno == errno.EADDRNOTAVAIL:
        return "AddressNotAvailable"

    if isinstance(e, ConnectionResetError):
        return "ConnectionResetByPeer"

    if isinstance(e, ConnectionAbortedError):
        return "ConnectionAborted"

    if isinstance(e, OSError) and e.errno == errno.EINVAL:
        # XXX: the exception is generic, but the only encountered
        # case for this is IPv4-mapped (::ffff:0:0/96) in AAAA
        return "DnsIpv4MappedInAaaa"

    if isinstance(e, aiohttp.http_exceptions.BadHttpMessage):
        return "BadHttp"

    if e.__cause__:
        return _classify_exception(e.__cause__)

    return None


def classify_exception(e: BaseException, url: str) -> str:
    if (name := _classify_exception(e)) is not None:
        return name

    print("Cannot classify error when checking {}:".format(url), file=sys.stderr)
    _print_exception_info(e)
    print("Traceback:", file=sys.stderr)
    traceback.print_stack()

    return "UnknownError"


@dataclass_json
@dataclass
class HttpResponse:
    status: str
    location: Optional[str] = None
    is_cloudflare: bool = False

    @staticmethod
    def from_http_response(
        url: str, response: aiohttp.ClientResponse
    ) -> "HttpResponse":
        return HttpResponse(str(response.status), response.headers.get("Location"), response.headers.get("server") == "cloudflare")

    @staticmethod
    def from_exception(url: str, exception: Exception) -> "HttpResponse":
        return HttpResponse(classify_exception(exception, url))


@dataclass_json
@dataclass
class Response:
    request_id: int
    response: HttpResponse


class FakeResolver(AbstractResolver):
    _address: str

    def __init__(self, address: str) -> None:
        self._address = address

    async def resolve(
        self, host: str, port: int = 0, family: socket.AddressFamily = socket.AF_INET
    ) -> List[ResolveResult]:
        return [
            {
                "hostname": host,
                "host": self._address,
                "port": port,
                "family": family,
                "proto": 0,
                "flags": socket.AI_NUMERICHOST,
            }
        ]

    async def close(self) -> None:
        pass


async def check_url(args: argparse.Namespace, request: Request) -> HttpResponse:
    family = socket.AF_INET6 if ":" in request.address else socket.AF_INET
    resolver = FakeResolver(request.address)
    connector = aiohttp.TCPConnector(
        resolver=resolver, use_dns_cache=False, limit_per_host=1, family=family
    )
    timeout = aiohttp.ClientTimeout(request.timeout)
    headers = {"User-Agent": args.user_agent}
    ssl_context = (
        ssl.SSLContext(protocol=ssl.PROTOCOL_TLSv1_2) if args.strict_ssl else True
    )

    async with aiohttp.ClientSession(
        cookie_jar=aiohttp.DummyCookieJar(),
        headers=headers,
        connector=connector,
    ) as session:
        do_request = session.get if request.use_get else session.head
        async with do_request(
            request.url, allow_redirects=False, ssl=ssl_context, timeout=timeout
        ) as http_response:
            return HttpResponse.from_http_response(request.url, http_response)


async def check_url_task(args: argparse.Namespace, request: Request):
    try:
        http_response = await check_url(args, request)
    except Exception as e:
        http_response = HttpResponse.from_exception(request.url, e)

    response = Response(
        request_id=request.request_id,
        response=http_response,
    )
    print(response.to_json(), flush=True)  # type: ignore


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )
    parser.add_argument(
        "--user-agent", required=True, type=str, help="HTTP user agent to use"
    )
    parser.add_argument(
        "--strict-ssl",
        action="store_true",
        help="stricter SSL requirements (require TLS1.2 support)",
    )
    return parser.parse_args()


async def main() -> None:
    args = parse_args()

    print("python process started", flush=True)

    loop = asyncio.get_event_loop()
    reader = asyncio.StreamReader()
    protocol = asyncio.StreamReaderProtocol(reader)
    await loop.connect_read_pipe(lambda: protocol, sys.stdin)

    try:
        async with asyncio.TaskGroup() as tg:
            async for line in reader:
                line = line.strip()
                request = Request.from_json(line)  # type: ignore
                tg.create_task(check_url_task(args, request))
    except (KeyboardInterrupt, CancelledError):
        print("interrupt/cancel signal received", file=sys.stderr)

    print("python process finishing", file=sys.stderr)


if __name__ == "__main__":
    asyncio.run(main())
