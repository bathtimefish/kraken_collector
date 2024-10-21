# Kraken Collector
Kraken Collector - The data collector function which is a part of the high level IoT router Kraken.

![logo](./assets/kraken-logo-300.png)

# Build & start 

```
sudo apt update
sudo apt install -y protobuf-compiler

cargo build
RUST_LOG=error,main=debug cargo run
```
