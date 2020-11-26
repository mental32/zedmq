import zmq
import os

ctx = zmq.Context()
s = ctx.socket(zmq.PULL)
s.bind(st := f"tcp://{os.environ['ADDRESS']}")

import time

while True:
    print(s.recv_multipart())
