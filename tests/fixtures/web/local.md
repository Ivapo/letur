# One image, from a server on this machine

`app/driver/drive.mjs` clause 6 opens this file against a server it starts
itself on `127.0.0.1:4446`, which serves `tests/fixtures/dot.png` and counts
what it is asked for. Nothing here reaches the internet, so the gate gives the
same answer on every machine — and it is plain `http://`, which the CLI accepts.

![a dot, fetched over the loopback](http://127.0.0.1:4446/dot.png)
