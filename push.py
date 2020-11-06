import zmq

ctx = zmq.Context()
s = ctx.socket(zmq.PUSH)
s.bind(st := "tcp://127.0.0.1:7890")

import time

while True:
    try:
        s.send(b"oof", flags=zmq.DONTWAIT)
    except zmq.error.Again:
        pass
    finally:
        print(f"oof! ({st})")
        time.sleep(1)
