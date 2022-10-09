# anna-rs client for WebAssembly

**wasmedge-anna-client** is a Rust client for [anna-rs] based on 
[Tokio for WasmEdge](https://github.com/WasmEdge/tokio). 
It communicates with [anna-rs] routing nodes and KVS nodes via vanilla 
TCP connections instead of Zenoh.

The **wasmedge-anna-client** can be compiled into
WebAssembly. The WebAssembly app can run inside the [WasmEdge Runtime](https://github.com/WasmEdge/WasmEdge#readme)
as a lightweight and secure alternative to natively compiled apps in Linux container.

[anna-rs]: https://github.com/essa-project/anna-rs

## Usage

```rust
use std::time::Duration;
use wasmedge_anna_client::{Client, ClientConfig};

let mut client = Client::new(ClientConfig {
    routing_ip: "127.0.0.1".parse().unwrap(),
    routing_port_base: 12340,
    routing_threads: 1,
    timeout: Duration::from_secs(10),
})?;

// put the value
client.put_lww("foo".into(), "bar".into()).await?;

// sleep 1 second
tokio::time::sleep(Duration::from_secs(1)).await;

// get the value
let bytes = client.get_lww("foo".into()).await?;
let value = String::from_utf8(bytes)?;
println!("Successfully GET value of `foo`: {}", value);
```

## Run the example

First, run routing node and KVS node of [anna-rs]:

```sh
$ cd anna-rs
$ cp example-config.yml config.yml
$ ANNA_PUBLIC_IP=127.0.0.1 ANNA_TCP_PORT_BASE=12340 cargo run --bin routing -- config.yml
$ # in another shell
$ ANNA_PUBLIC_IP=127.0.0.1 cargo run --bin kvs -- config.yml
```

Then, build and run the example app of **wasmedge-anna-client**:

```sh
$ cd example
$ cargo build --target wasm32-wasi
$ /path/to/wasmedge --dir .:. target/wasm32-wasi/debug/example.wasm
```

You can find more examples in [wasmedge-db-examples](https://github.com/WasmEdge/wasmedge-db-examples/tree/main/anna).

## Attribution

Many code of this driver is derived from [anna-rs].
