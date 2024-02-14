use blockstore::{Blockstore, BlockstoreError};
use cid::CidGeneric;

#[derive(Debug)]
pub struct NoBlockstore;

#[async_trait::async_trait]
impl Blockstore for NoBlockstore {
    async fn get<const S: usize>(
        &self,
        cid: &CidGeneric<S>,
    ) -> Result<Option<Vec<u8>>, BlockstoreError> {
        unimplemented!()
    }

    async fn put_keyed<const S: usize>(
        &self,
        cid: &CidGeneric<S>,
        data: &[u8],
    ) -> Result<(), BlockstoreError> {
        unimplemented!()
    }
}
