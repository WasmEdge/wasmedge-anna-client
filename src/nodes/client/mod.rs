//! New generation of client node that expose a GET/PUT-based interface to users.

use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use anna_api::{
    messages::{request::KeyOperation, response::ClientResponseValue},
    ClientKey,
};
use eyre::{eyre, Context, ContextCompat};
use futures::Future;
use rand::prelude::IteratorRandom;
use serde::{Deserialize, Serialize};
use tokio::{
    net::{tcp, TcpStream},
    sync::{oneshot, Mutex},
};

use crate::{
    messages::{AddressRequest, AddressResponse, Response, TcpMessage},
    nodes::{receive_tcp_message, send_tcp_message},
    topics::{ClientThread, KvsThread},
};

use self::{client_request::ClientRequest, transaction::ReadCommittedTransaction};

mod client_request;
pub mod redis_like;
mod transaction;

/// Configuration for [`Client`].
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// IP address of routing node.
    pub routing_ip: IpAddr,
    /// TCP port base of routing node.
    pub routing_port_base: u16,
    /// Number of threads used for routing.
    pub routing_threads: u32,
    /// Timeout for client requests.
    pub timeout: Duration,
}

/// Anna client.
pub struct Client {
    client_thread: ClientThread,
    routing_ip: IpAddr,
    routing_port_base: u16,
    routing_threads: u32,
    _timeout: Duration,
    next_request_id: u32,
    key_address_cache: HashMap<ClientKey, HashSet<KvsThread>>,
    kvs_tcp_address_cache: HashMap<KvsThread, SocketAddr>,
    tcp_write_halves: HashMap<SocketAddr, Arc<Mutex<tcp::OwnedWriteHalf>>>,
    address_response_promises:
        Arc<Mutex<HashMap<String /* request_id */, oneshot::Sender<AddressResponse>>>>,
    response_promises: Arc<Mutex<HashMap<String /* request_id */, oneshot::Sender<Response>>>>,
}

struct ThisClient {
    address_response_promises: Arc<Mutex<HashMap<String, oneshot::Sender<AddressResponse>>>>,
    response_promises: Arc<Mutex<HashMap<String, oneshot::Sender<Response>>>>,
}

impl ThisClient {
    fn from(client: &Client) -> Self {
        Self {
            address_response_promises: client.address_response_promises.clone(),
            response_promises: client.response_promises.clone(),
        }
    }
}

impl Client {
    /// Create a new client node.
    pub fn new(config: ClientConfig) -> eyre::Result<Self> {
        assert!(config.routing_threads > 0);
        let client_thread = ClientThread::new(format!("client-{}", uuid::Uuid::new_v4()), 0);

        Ok(Self {
            client_thread,
            routing_ip: config.routing_ip,
            routing_port_base: config.routing_port_base,
            routing_threads: config.routing_threads,
            _timeout: config.timeout,
            next_request_id: 1,
            kvs_tcp_address_cache: Default::default(),
            key_address_cache: Default::default(),
            tcp_write_halves: Default::default(),
            address_response_promises: Default::default(),
            response_promises: Default::default(),
        })
    }

    fn gen_request_id(&mut self) -> String {
        let id = format!(
            "{}:{}_{}",
            self.client_thread.node_id, self.client_thread.thread_id, self.next_request_id
        );
        log::trace!("Generated request ID: {}", id);
        self.next_request_id = (self.next_request_id + 1) % 10000;
        id
    }

    fn make_address_request(&mut self, key: ClientKey) -> AddressRequest {
        log::trace!("Making AddressRequest for key: {:?}", key);
        AddressRequest {
            request_id: self.gen_request_id(),
            response_address: self
                .client_thread
                .address_response_topic("anna")
                .to_string(),
            keys: vec![key],
        }
    }

    fn make_request(&mut self, operation: KeyOperation) -> ClientRequest {
        log::trace!("Making ClientRequest for operation: {:?}", operation);
        ClientRequest {
            operation,
            response_address: self.client_thread.response_topic("anna").to_string(),
            request_id: self.gen_request_id(),
            address_cache_size: HashMap::new(),
            timestamp: Instant::now(),
        }
    }

    async fn make_address_response_promise(
        &mut self,
        request_id: String,
    ) -> impl Future<Output = eyre::Result<AddressResponse>> {
        let (tx, rx) = oneshot::channel();
        self.address_response_promises
            .lock()
            .await
            .insert(request_id, tx);
        async { rx.await.map_err(Into::into) }
    }

    async fn make_response_promise(
        &mut self,
        request_id: String,
    ) -> impl Future<Output = eyre::Result<Response>> {
        let (tx, rx) = oneshot::channel();
        self.response_promises.lock().await.insert(request_id, tx);
        async { rx.await.map_err(Into::into) }
    }

    fn get_routing_thread_id(&self) -> u32 {
        use rand::prelude::*;

        let mut rng = rand::thread_rng();

        let thread_id = (0..self.routing_threads).choose(&mut rng).unwrap_or(0);
        log::trace!("Selected routing thread_id: {:?}", thread_id);
        thread_id
    }

    fn get_routing_tcp_address(&self) -> SocketAddr {
        let routing_thread_id = self.get_routing_thread_id();
        SocketAddr::new(
            self.routing_ip,
            self.routing_port_base + routing_thread_id as u16,
        )
    }

    async fn loop_receiving_tcp_message(
        this: ThisClient,
        mut reader: tcp::OwnedReadHalf,
    ) -> eyre::Result<()> {
        loop {
            // TODO: handle error
            let message = receive_tcp_message(&mut reader).await?;
            if let Some(message) = message {
                match message {
                    TcpMessage::AddressResponse(response) => {
                        if let Some(tx) = this
                            .address_response_promises
                            .lock()
                            .await
                            .remove(&response.response_id)
                        {
                            tx.send(response).unwrap();
                        } else {
                            // TODO: update address cache
                            log::warn!("Unexpected AddressResponse: {:?}", response);
                        }
                    }
                    TcpMessage::Response(response) => {
                        if let Some(response_id) = response.response_id.as_ref() {
                            if let Some(tx) =
                                this.response_promises.lock().await.remove(response_id)
                            {
                                tx.send(response).unwrap();
                            }
                        } else {
                            log::warn!("Unexpected Response: {:?}", response);
                        }
                    }
                    other => panic!("unexpected tcp message {:?}", other),
                }
            }
        }
        // TODO: recycle dead connection
    }

    async fn get_tcp_writer(
        &mut self,
        addr: SocketAddr,
    ) -> eyre::Result<Arc<Mutex<tcp::OwnedWriteHalf>>> {
        Ok(match self.tcp_write_halves.entry(addr) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.get().clone(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                log::trace!("Connecting TCP to address: {:?}", addr);
                let stream = TcpStream::connect(addr)
                    .await
                    .context("failed to connect to tcp stream")?;
                stream
                    .set_nodelay(true)
                    .context("failed to set nodelay for tcpstream")?;
                let (reader, writer) = stream.into_split();
                let writer = entry.insert(Arc::new(Mutex::new(writer))).clone();
                tokio::spawn(Self::loop_receiving_tcp_message(
                    ThisClient::from(self),
                    reader,
                ));
                writer
            }
        })
    }

    async fn send_tcp_message(
        &mut self,
        addr: SocketAddr,
        message: TcpMessage,
    ) -> eyre::Result<()> {
        let writer = self.get_tcp_writer(addr).await?;
        let mut writer = writer.lock().await;
        send_tcp_message(&message, &mut writer).await
    }

    async fn send_address_request(
        &mut self,
        request: AddressRequest,
    ) -> eyre::Result<AddressResponse> {
        let request_id = request.request_id.clone();
        let addr = self.get_routing_tcp_address();
        let promise = self.make_address_response_promise(request_id).await;
        self.send_tcp_message(addr, TcpMessage::AddressRequest(request))
            .await?;
        promise.await.map_err(Into::into)
    }

    fn handle_address_response(&mut self, response: AddressResponse) -> eyre::Result<()> {
        response
            .tcp_sockets
            .into_iter()
            .for_each(|(kvs_thread, addr)| {
                self.kvs_tcp_address_cache.insert(kvs_thread, addr);
            });

        for key_addr in response.addresses {
            let key = key_addr.key;
            for node in key_addr.nodes {
                self.key_address_cache
                    .entry(key.clone())
                    .or_default()
                    .insert(node);
            }
        }

        Ok(())
    }

    /// Make and send an AddressRequest for the given key,
    /// and update the address cache with the response.
    async fn query_key_address(&mut self, key: &ClientKey) -> eyre::Result<()> {
        log::trace!("Querying address for key: {:?}", key);
        let request = self.make_address_request(key.clone());
        let response = self.send_address_request(request).await?;
        assert!(response.error.is_none()); // TODO: handle the error (cache invalidation, no server, etc.)
        self.handle_address_response(response)?;
        Ok(())
    }

    fn get_kvs_thread_from_cache(&self, key: &ClientKey) -> Option<KvsThread> {
        let mut rng = rand::thread_rng();
        let addr_set = self.key_address_cache.get(key);
        if let Some(addr_set) = addr_set {
            addr_set.iter().choose(&mut rng).cloned()
        } else {
            None
        }
    }

    async fn get_kvs_thread(&mut self, key: &ClientKey) -> eyre::Result<Option<KvsThread>> {
        let thread = match self.get_kvs_thread_from_cache(key) {
            thread @ Some(_) => thread, // cache hit
            None => {
                // cache miss
                self.query_key_address(key).await?;
                self.get_kvs_thread_from_cache(key)
            }
        };
        log::trace!("Selected kvs thread: {:?}, key: {:?}", thread, key);
        Ok(thread)
    }

    async fn get_key_tcp_address(&mut self, key: &ClientKey) -> eyre::Result<Option<SocketAddr>> {
        let kvs_thread = match self.get_kvs_thread(key).await? {
            Some(thread) => thread,
            None => return Ok(None),
        };
        let addr = match self.kvs_tcp_address_cache.get(&kvs_thread) {
            addr @ Some(_) => addr, // cache hit
            None => {
                // cache miss
                self.query_key_address(key).await?;
                self.kvs_tcp_address_cache.get(&kvs_thread)
            }
        }
        .cloned();
        log::trace!("Got kvs tcp address: {:?}, thread: {:?}", addr, kvs_thread);
        Ok(addr)
    }

    async fn send_request(&mut self, request: ClientRequest) -> eyre::Result<Response> {
        let request_id = request.request_id.clone();
        let key = request.operation.key();
        let addr = self
            .get_key_tcp_address(&key)
            .await?
            .context("fail to get tcp address of the kvs thread the key locates")?;
        let promise = self.make_response_promise(request_id).await;
        self.send_tcp_message(addr, TcpMessage::Request(request.into()))
            .await?;
        promise.await.map_err(Into::into)
    }

    async fn get_lattice(&mut self, key: ClientKey) -> eyre::Result<ClientResponseValue> {
        let request = self.make_request(KeyOperation::Get(key));
        let mut response = self.send_request(request).await?;

        // TODO: handle cache invalidation and other special errors
        response.error?;

        let response_tuple = response
            .tuples
            .pop()
            .ok_or_else(|| eyre!("response has no tuples"))?;

        if let Some(error) = response_tuple.error {
            Err(error.into())
        } else {
            response_tuple.lattice.context("expected lattice value")
        }
    }

    /// Try to put a *last writer wins* value with the given key.
    pub async fn put_lww(&mut self, key: ClientKey, value: Vec<u8>) -> eyre::Result<()> {
        let request = self.make_request(KeyOperation::Put(key, value));
        let response = self.send_request(request).await?;
        response.error?;
        Ok(())
    }

    /// Try to get a *last writer wins* value with the given key.
    pub async fn get_lww(&mut self, key: ClientKey) -> eyre::Result<Vec<u8>> {
        let value = self.get_lattice(key).await?;

        match value {
            ClientResponseValue::Bytes(bytes) => Ok(bytes),
            other => Err(eyre::anyhow!("expected bytes, got `{:?}`", other)),
        }
    }

    /// Begin a transaction that satisfies *read committed* isolation level.
    pub fn begin_transaction(&mut self) -> ReadCommittedTransaction {
        ReadCommittedTransaction::new(self)
    }

    /// Try to merge a set value with the given key.
    pub async fn add_set(&mut self, key: ClientKey, set: HashSet<Vec<u8>>) -> eyre::Result<()> {
        let request = self.make_request(KeyOperation::SetAdd(key, set));
        let response = self.send_request(request).await?;
        response.error?;
        Ok(())
    }

    /// Try to get a set value with the given key.
    pub async fn get_set(&mut self, key: ClientKey) -> eyre::Result<HashSet<Vec<u8>>> {
        let value = self.get_lattice(key).await?;

        match value {
            ClientResponseValue::Set(set) => Ok(set),
            other => Err(eyre::anyhow!("expected Set lattice, got `{:?}`", other)),
        }
    }

    /// Try to merge a hashmap value with the given key.
    pub async fn add_map(
        &mut self,
        key: ClientKey,
        map: HashMap<String, Vec<u8>>,
    ) -> eyre::Result<()> {
        let request = self.make_request(KeyOperation::MapAdd(key, map));
        let response = self.send_request(request).await?;
        response.error?;
        Ok(())
    }

    /// Try to get a hashmap value with the given key.
    pub async fn get_map(&mut self, key: ClientKey) -> eyre::Result<HashMap<String, Vec<u8>>> {
        let value = self.get_lattice(key).await?;

        match value {
            ClientResponseValue::Map(set) => Ok(set),
            other => Err(eyre::anyhow!("expected Set lattice, got `{:?}`", other)),
        }
    }

    /// Try to Increase int value with the given key.
    pub async fn inc(&mut self, key: ClientKey, value: i64) -> eyre::Result<i64> {
        let request = self.make_request(KeyOperation::Inc(key, value));
        let mut response = self.send_request(request).await?;
        // TODO: handle cache invalidation and other special errors
        response.error?;

        let response_tuple = response
            .tuples
            .pop()
            .ok_or_else(|| eyre!("response has no tuples"))?;

        if let Some(error) = response_tuple.error {
            Err(error.into())
        } else {
            let value = response_tuple.lattice.context("expected lattice value")?;
            match value {
                ClientResponseValue::Int(v) => Ok(v),
                other => Err(eyre::anyhow!("expected Set lattice, got `{:?}`", other)),
            }
        }
    }
}
