#![warn(missing_docs)]

//! Rust port of the [`anna`](https://github.com/hydro-project/anna) key-value store, using
//! [`zenoh`](https://zenoh.io/) for communication.
//!
//! ## Usage Example
//!
//! Open four terminal windows and run the following commands in them (one per terminal window):
//!
//! 1. `cargo run --bin logger` to start the zenoh logger, so that we can see the messages that
//!    are sent. This step is optional.
//! 2. `cargo run --bin routing -- example-config.yml` to start the
//!    [routing node](nodes::RoutingNode).
//! 3. `cargo run --bin kvs -- example-config.yml` to start the
//!    [key-value store node](nodes::KvsNode).
//! 4. `cargo run --bin client -- example-config.yml` to start the
//!    [client proxy node](nodes::ClientNode).
//!
//! The client proxy executable will show a `kvs>` prompt, in which you can use various commands
//! such as `put <key> <value>` to write a value or `get <key>` to retrieve a stored value. See
//! the [`ClientNode::run_interactive`][nodes::ClientNode::run_interactive] method for a full
//! list of supported commands, including background information on the used lattice types.

pub use anna_api::*;

pub mod nodes;
pub use nodes::*;
