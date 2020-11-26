import zmq
import os

ctx = zmq.Context()

s = ctx.socket(zmq.PUSH)
s.bind(st := f"tcp://{os.environ['ADDRESS']}")

import time

while True:
    try:
        s.send(b"oof", flags=zmq.DONTWAIT)
    except zmq.error.Again:
        pass
    finally:
        print(f"oof! ({st})")
        time.sleep(1)
