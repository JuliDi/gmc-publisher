# gmc-logger
A minimal application for uploading the Geiger Counter's current CPM values to gmcmap.com.

## Features
* Only specify port, user ID and Geiger Counter ID – and you are good to go
* Minimal dependencies
* Easy to adapt to specifc needs or other models

## Dependencies
* [clap](https://crates.io/crates/clap): for parsing command line arguments
* [ureq](https://crates.io/crates/ureq): for performing the http GET request to gmcmap.com
* [serialport](https://crates.io/crates/serialport): for interfacing with the GMC's serial port via USB

## Installation
### Compilation
* for your machine:

  `cargo build --release`
* for a different platform (e.g. 32 bit Linux) using [cross](https://github.com/rust-embedded/cross):

  `cross build --target i686-unknown-linux-musl --release`

### Running gmc-logger

```
USAGE:
    gmc-logger --aid <AID> --gid <GID> --port <PORT>

FLAGS:
    -h, --help    Prints help information

OPTIONS:
    -a, --aid <AID>      The gmcmap.com user account ID
    -g, --gid <GID>      The gmcmap.com Geiger Counter ID
    -p, --port <PORT>    The device path to a serial port, e. g. /dev/tty.USB0
```

Example: `gmc-logger --port=/dev/ttyUSB0 --aid=12345 --gid=1234567890`