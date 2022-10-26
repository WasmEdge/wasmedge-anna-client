use std::collections::HashMap;

use anna_api::{
    lattice::{last_writer_wins::Timestamp, LastWriterWinsLattice},
    ClientKey, LatticeValue,
};

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

    pub async fn get(&mut self, key: ClientKey) -> eyre::Result<Vec<u8>> {
        if let Some(value) = self.write_buffer.get(&key) {
            Ok(value.clone())
        } else {
            self.client.get_lww(key).await
        }
    }

    pub async fn put(&mut self, key: ClientKey, value: Vec<u8>) -> eyre::Result<()> {
        self.write_buffer.insert(key, value);
        Ok(())
    }

    pub async fn commit(self) -> eyre::Result<()> {
        let commit_time = Timestamp::now();
        for (key, value) in self.write_buffer.into_iter() {
            self.client
                .put_lattice(
                    key,
                    LatticeValue::Lww(LastWriterWinsLattice::from_pair(commit_time, value)),
                )
                .await?;
        }
        Ok(())
    }
}
