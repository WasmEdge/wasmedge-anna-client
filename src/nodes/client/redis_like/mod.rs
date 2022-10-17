//! Provides Redis-like [`Client`], [`Connection`] and operations, etc.

use anna_api::ClientKey;

use crate::ClientConfig;

use self::convert::ToAnnaValue;

mod convert;

/// Redis-like client.
pub struct Client {
    config: ClientConfig,
}

impl Client {
    /// Creates a new client with given configuration.
    pub fn open(config: ClientConfig) -> eyre::Result<Self> {
        Ok(Self { config })
    }

    /// Get an async connection object.
    pub async fn get_async_connection(&self) -> eyre::Result<Connection> {
        let client = crate::Client::new(self.config.clone())?;
        Ok(Connection { client })
    }
}

/// Async Redis-like connection to Anna cluster.
pub struct Connection {
    client: crate::Client,
}

impl Connection {
    pub async fn get<K>(&mut self, key: K) -> eyre::Result<Vec<u8>>
    where
        K: Into<ClientKey>,
    {
        self.client.get_lww(key.into()).await
    }

    pub async fn set<K, V>(&mut self, key: K, value: V) -> eyre::Result<()>
    where
        K: Into<ClientKey>,
        V: ToAnnaValue,
    {
        self.client.put_lww(key.into(), value.to_anna_value()).await
    }
}
