"""S06 request validation over the existing bounded S05 transport lifecycle."""
import os
import selectors
import time
from s05_protocol import Process as BoundedProcess
from s06_protocol import MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, StreamCase, canonical, validate_request


class Process(BoundedProcess):
    def __init__(self, command, stderr_path, env=None, deadline=60):
        self.request_expires = None
        try:
            super().__init__(command, stderr_path, env, deadline, MAX_RESPONSE_BYTES)
        except BaseException:
            if hasattr(self, "process"):
                self.close()
            elif hasattr(self, "log"):
                self.log.close()
            raise

    def wait(self, fd, event, expires):
        if self.request_expires is not None:
            expires = min(expires, self.request_expires)
        with selectors.DefaultSelector() as selector:
            selector.register(fd, event)
            remaining = expires-time.monotonic()
            if remaining <= 0 or not selector.select(remaining):
                raise TimeoutError("S06 adapter record or request deadline exceeded")

    def check_request_deadline(self):
        if self.request_expires is not None and time.monotonic() >= self.request_expires:
            raise TimeoutError("S06 adapter request deadline exceeded")

    def read(self, allow_eof=False):
        # Buffered records do not enter wait(), so check on both sides of JSON
        # decoding too. Progress on individual records cannot renew a request.
        self.check_request_deadline()
        record = super().read(allow_eof)
        self.check_request_deadline()
        return record

    def send(self, request):
        validate_request(request)
        if self.request_expires is not None:
            raise RuntimeError("previous S06 request has not completed")
        raw = canonical(request)+b"\n"
        if len(raw)-1 > MAX_REQUEST_BYTES:
            raise ValueError("oversized S06 request")
        view = memoryview(raw)
        expires = time.monotonic()+self.deadline
        self.request_expires = expires
        while view:
            self.wait(self.process.stdin, selectors.EVENT_WRITE, expires)
            try:
                written = os.write(self.process.stdin.fileno(), view)
            except BlockingIOError:
                continue
            view = view[written:]

    def observations(self, request):
        state = StreamCase(request)
        while not state.ended:
            record = self.read()
            state.accept(record)
            if state.ended:
                self.request_expires = None
            yield record
