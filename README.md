# ssh-control

Interface to SSH master socket.

## Protocol
The protocol is described [here](https://cvsweb.openbsd.org/src/usr.bin/ssh/PROTOCOL.mux?annotate=HEAD).

Long story short, here the few things to know:

* Intergers are encoded in big-endian.
* Booleans are 32 intergers.
* Strings are encoded with their length first (32 bits interger), then their data (no null
  terminator)
* Terminating `0` (for environment string in MUX_C_NEW_SESSION for instance) is litteraly a null
  byte.
