use std::collections::HashMap;

use anna_api::ClientKey;

use crate::Client;

pub struct ReadCommittedTransaction<'a> {
    client: &'a mut Client,
    write_buffer: HashMap<ClientKey, Vec<u8>>,
}

impl<'a> ReadCommittedTransaction<'a> {
    pub(crate) fn new(client: &'a mut Client) -> Self {
        Self {
            client,
            write_buffer: HashMap::new(),
        }
    }

    pub async fn get_lww(&mut self, key: ClientKey) -> eyre::Result<Vec<u8>> {
        if let Some(value) = self.write_buffer.get(&key) {
            Ok(value.clone())
        } else {
            self.client.get_lww(key).await
        }
    }

    pub async fn put_lww(&mut self, key: ClientKey, value: Vec<u8>) -> eyre::Result<()> {
        self.write_buffer.insert(key, value);
        Ok(())
    }

    pub async fn commit(self) -> eyre::Result<()> {
        for (key, value) in self.write_buffer.into_iter() {
            self.client.put_lww(key, value).await?;
        }
        Ok(())
    }
}
