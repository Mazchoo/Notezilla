"""Pub/sub for filesystem change pings to the GUI."""

from __future__ import annotations

import asyncio
import threading
from typing import Callable, List

from starlette.requests import Request
from starlette.responses import StreamingResponse

from src.backend.logger import LOGGER
from src.config import FILE_CHANGE_SSE_IDLE_TIMEOUT_SECONDS

Subscriber = Callable[[str], None]

FILE_CHANGE_SUBSCRIBERS_LOCK = threading.Lock()
FILE_CHANGE_SUBSCRIBERS: List[Subscriber] = []


def subscribe(callback: Subscriber) -> Callable[[], None]:
    """Register a payload callback. Return an unsubscribe function."""
    with FILE_CHANGE_SUBSCRIBERS_LOCK:
        FILE_CHANGE_SUBSCRIBERS.append(callback)

    def unsubscribe():
        with FILE_CHANGE_SUBSCRIBERS_LOCK:
            try:
                FILE_CHANGE_SUBSCRIBERS.remove(callback)
            except ValueError:
                pass

    return unsubscribe


def publish():
    """Ping every subscriber that a watcher batch was processed."""
    with FILE_CHANGE_SUBSCRIBERS_LOCK:
        subscribers = list(FILE_CHANGE_SUBSCRIBERS)
    for callback in subscribers:
        try:
            callback("{}")  # empty payload
        except Exception:  # pylint: disable=broad-except
            LOGGER.exception("File-change subscriber failed")


def clear_subscribers():
    """Drop every subscriber. Tests use this to isolate cases."""
    with FILE_CHANGE_SUBSCRIBERS_LOCK:
        FILE_CHANGE_SUBSCRIBERS.clear()


async def file_change_sse(request: Request) -> StreamingResponse:
    """Stream file-change pings until the client disconnects."""
    queue: asyncio.Queue[str] = asyncio.Queue()
    loop = asyncio.get_running_loop()

    def on_payload(payload: str):
        loop.call_soon_threadsafe(queue.put_nowait, payload)

    unsubscribe = subscribe(on_payload)

    async def generate():
        try:
            yield ": connected\n\n"
            while True:
                if await request.is_disconnected():
                    break
                try:
                    payload = await asyncio.wait_for(
                        queue.get(), timeout=FILE_CHANGE_SSE_IDLE_TIMEOUT_SECONDS
                    )
                    yield f"data: {payload}\n\n"
                except asyncio.TimeoutError:
                    yield ": keepalive\n\n"
        finally:
            unsubscribe()

    return StreamingResponse(
        generate(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
            "Access-Control-Allow-Origin": "*",
        },
    )
