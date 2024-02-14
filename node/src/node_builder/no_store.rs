use celestia_types::hash::Hash;
use celestia_types::ExtendedHeader;
use cid::Cid;

use crate::store::{SamplingMetadata, Store, StoreError};

#[derive(Debug)]
pub struct NoStore;

#[async_trait::async_trait]
impl Store for NoStore {
    async fn get_head(&self) -> Result<ExtendedHeader, StoreError> {
        unimplemented!()
    }

    async fn get_by_hash(&self, hash: &Hash) -> Result<ExtendedHeader, StoreError> {
        unimplemented!()
    }

    async fn get_by_height(&self, height: u64) -> Result<ExtendedHeader, StoreError> {
        unimplemented!()
    }

    async fn head_height(&self) -> Result<u64, StoreError> {
        unimplemented!()
    }

    async fn has(&self, hash: &Hash) -> bool {
        unimplemented!()
    }

    async fn has_at(&self, height: u64) -> bool {
        unimplemented!()
    }

    async fn append_single_unchecked(&self, header: ExtendedHeader) -> Result<(), StoreError> {
        unimplemented!()
    }

    async fn next_unsampled_height(&self) -> Result<u64, StoreError> {
        unimplemented!()
    }

    async fn update_sampling_metadata(
        &self,
        height: u64,
        accepted: bool,
        cids: Vec<Cid>,
    ) -> Result<u64, StoreError> {
        unimplemented!()
    }

    async fn get_sampling_metadata(
        &self,
        height: u64,
    ) -> Result<Option<SamplingMetadata>, StoreError> {
        unimplemented!()
    }
}
