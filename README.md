# ssh-control

Interface to SSH master socket.

## Protocol
The protocol is described [here](https://cvsweb.openbsd.org/src/usr.bin/ssh/PROTOCOL.mux?annotate=HEAD).

Long story short, here the few things to know:

* Integers are encoded in big-endian.
* Booleans are 32 integers.
* Strings are encoded with their length first (32 bits integer), then their data (no null
  terminator)
* Terminating `0` (for environment string in MUX_C_NEW_SESSION for instance) is litteraly a null
  byte.

## Build

```bash
git clone --recurse https://github.com/mephesto1337/ssh-control
cargo build --release
```

## Examples

see [src/main.rs](./src/main.rs).
