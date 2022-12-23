//! Provides Redis-like [`Client`], [`Connection`] and operations, etc.

use std::collections::{HashMap, HashSet};

use anna_api::{AnnaError, ClientKey};

use crate::ClientConfig;

use self::convert::{FromAnnaValue, ToAnnaValue};

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
    /// GET key
    pub async fn get<K, V>(&mut self, key: K) -> eyre::Result<V>
    where
        K: Into<ClientKey>,
        V: FromAnnaValue,
    {
        let value = self.client.get_lww(key.into()).await?;
        V::from_anna_value(&value)
    }

    /// SET key value
    pub async fn set<K, V>(&mut self, key: K, value: V) -> eyre::Result<()>
    where
        K: Into<ClientKey>,
        V: ToAnnaValue,
    {
        self.client.put_lww(key.into(), value.to_anna_value()).await
    }

    /// SETNX key value
    pub async fn set_nx<K, V>(&mut self, key: K, value: V) -> eyre::Result<()>
    where
        K: Into<ClientKey>,
        V: ToAnnaValue,
    {
        let key = key.into();
        let mut tx = self.client.begin_transaction();
        let res = tx.get_lww(key.clone()).await;
        if res.is_ok() {
            // exists
            return Ok(());
        }
        let err_report = res.err().unwrap();
        if let Some(AnnaError::KeyDoesNotExist) = err_report.downcast_ref() {
            tx.put_lww(key, value.to_anna_value()).await?;
            tx.commit().await
        } else {
            Err(err_report)
        }
    }

    /// SGET key
    pub async fn s_get<K>(&mut self, key: K) -> eyre::Result<HashSet<Vec<u8>>>
    where
        K: Into<ClientKey>,
    {
        self.client.get_set(key.into()).await
    }

    /// SADD key value
    pub async fn s_add<K>(&mut self, key: K, value: HashSet<Vec<u8>>) -> eyre::Result<()>
    where
        K: Into<ClientKey>,
    {
        self.client.add_set(key.into(), value).await
    }

    /// HGETALL key
    pub async fn h_getall<K>(&mut self, key: K) -> eyre::Result<HashMap<String, Vec<u8>>>
    where
        K: Into<ClientKey>,
    {
        self.client.get_map(key.into()).await
    }

    /// HMSET key field1 value1 [field2 value2 ]
    pub async fn h_mset<K>(&mut self, key: K, value: HashMap<String, Vec<u8>>) -> eyre::Result<()>
    where
        K: Into<ClientKey>,
    {
        self.client.add_map(key.into(), value).await
    }

    /// INC key value
    pub async fn inc<K>(&mut self, key: K, value: i64) -> eyre::Result<i64>
    where
        K: Into<ClientKey>,
    {
        self.client.inc(key.into(), value).await
    }
}
