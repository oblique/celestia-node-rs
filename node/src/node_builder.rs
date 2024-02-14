use std::io;
use std::pin::Pin;
use std::sync::Arc;

use blockstore::{Blockstore, BlockstoreError};
use celestia_types::hash::Hash;
use futures::future::BoxFuture;
use libp2p::identity::Keypair;
use libp2p::Multiaddr;

pub(crate) mod no_blockstore;
pub(crate) mod no_store;

use crate::blockstore::SledBlockstore;
use crate::network::Network;
use crate::node::Node;
use crate::p2p::{P2p, P2pArgs, P2pError};
use crate::store::{SledStore, Store, StoreError};
use crate::syncer::{Syncer, SyncerArgs, SyncerError};

use self::no_blockstore::NoBlockstore;
use self::no_store::NoStore;

type Result<T, E = NodeBuilderError> = std::result::Result<T, E>;

/// Representation of all the errors that can occur when interacting with the [`NodeBuilder`].
#[derive(Debug, thiserror::Error)]
pub enum NodeBuilderError {
    /// An error propagated from the [`P2p`] module.
    #[error(transparent)]
    P2p(#[from] P2pError),

    /// An error propagated from the [`Syncer`] module.
    #[error(transparent)]
    Syncer(#[from] SyncerError),

    /// An error propagated from the [`Blockstore`] module.
    #[error(transparent)]
    BlockstoreError(#[from] BlockstoreError),

    /// An error propagated from the [`Store`] module.
    #[error(transparent)]
    StoreError(#[from] StoreError),

    /// An error propagated from the IO operation.
    #[error("Received io error from persistent storage: {0}")]
    IoError(#[from] io::Error),

    /// Network was required but not provided.
    #[error("Network not provided. Consider calling `.with_network`")]
    NetworkMissing,
}

/// Node conifguration.
pub struct NodeBuilder<B, S>
where
    B: Blockstore,
    S: Store,
{
    /// An id of the network to connect to.
    network: Option<Network>,
    /// The hash of the genesis block in network.
    genesis_hash: Option<Hash>,
    /// The keypair to be used as [`Node`]s identity.
    p2p_local_keypair: Option<Keypair>,
    /// List of bootstrap nodes to connect to and trust.
    p2p_bootnodes: Vec<Multiaddr>,
    /// List of the addresses where [`Node`] will listen for incoming connections.
    p2p_listen_on: Vec<Multiaddr>,
    /// The blockstore for bitswap.
    blockstore: Option<StoreKind<B>>,
    /// The store for headers.
    store: Option<StoreKind<S>>,
}

enum StoreKind<S> {
    #[cfg(not(target_arch = "wasm32"))]
    InitWithSledDb(Box<dyn FnOnce(sled::Db) -> BoxFuture<'static, Result<S, NodeBuilderError>>>),
    Value(S),
}

impl NodeBuilder<NoBlockstore, NoStore> {
    pub fn new() -> Self {
        Self {
            network: None,
            genesis_hash: None,
            p2p_local_keypair: None,
            p2p_bootnodes: vec![],
            p2p_listen_on: vec![],
            blockstore: None,
            store: None,
        }
    }
}

impl<B, S> NodeBuilder<B, S>
where
    B: Blockstore + 'static,
    S: Store,
{
    pub fn with_network(mut self, network: Network) -> Self {
        self.network = Some(network);
        self
    }

    pub fn with_genesis(mut self, hash: Option<Hash>) -> Self {
        self.genesis_hash = hash;
        self
    }

    pub fn with_p2p_keypair(mut self, keypair: Keypair) -> Self {
        self.p2p_local_keypair = Some(keypair);
        self
    }

    pub fn with_listeners(mut self, listeners: Vec<Multiaddr>) -> Self {
        self.p2p_listen_on = listeners;
        self
    }

    pub fn with_bootnodes(mut self, bootnodes: Vec<Multiaddr>) -> Self {
        self.p2p_bootnodes = bootnodes;
        self
    }

    pub fn with_blockstore<NEW_B>(self, blockstore: NEW_B) -> NodeBuilder<NEW_B, S>
    where
        NEW_B: Blockstore,
    {
        NodeBuilder {
            network: self.network,
            genesis_hash: self.genesis_hash,
            p2p_local_keypair: self.p2p_local_keypair,
            p2p_bootnodes: self.p2p_bootnodes,
            p2p_listen_on: self.p2p_listen_on,
            blockstore: Some(StoreKind::Value(blockstore)),
            store: self.store,
        }
    }

    pub fn with_store<NEW_S>(self, store: NEW_S) -> NodeBuilder<B, NEW_S>
    where
        NEW_S: Store,
    {
        NodeBuilder {
            network: self.network,
            genesis_hash: self.genesis_hash,
            p2p_local_keypair: self.p2p_local_keypair,
            p2p_bootnodes: self.p2p_bootnodes,
            p2p_listen_on: self.p2p_listen_on,
            blockstore: self.blockstore,
            store: Some(StoreKind::Value(store)),
        }
    }

    pub async fn build(self) -> Result<Node<S>> {
        let network = self.network.ok_or(NodeBuilderError::NetworkMissing)?;
        let genesis_hash = self.genesis_hash.or_else(|| network.genesis());

        let bootnodes = if self.p2p_bootnodes.is_empty() {
            network.canonical_bootnodes().collect()
        } else {
            self.p2p_bootnodes
        };

        let store = self.store.expect("todo");
        let blockstore = self.blockstore.expect("todo");

        let local_keypair = self
            .p2p_local_keypair
            .unwrap_or_else(Keypair::generate_ed25519);

        #[cfg(not(target_arch = "wasm32"))]
        let sled_db = if matches!(blockstore, StoreKind::InitWithSledDb(_))
            || matches!(store, StoreKind::InitWithSledDb(_))
        {
            Some(native::default_sled_db(&network).await?)
        } else {
            None
        };

        let blockstore = match blockstore {
            #[cfg(not(target_arch = "wasm32"))]
            StoreKind::InitWithSledDb(f) => f(sled_db.clone().unwrap()).await?,
            StoreKind::Value(blockstore) => blockstore,
        };

        let store = match store {
            #[cfg(not(target_arch = "wasm32"))]
            StoreKind::InitWithSledDb(f) => f(sled_db.clone().unwrap()).await?,
            StoreKind::Value(store) => store,
        };

        let store = Arc::new(store);

        let p2p = Arc::new(P2p::start(P2pArgs {
            network,
            local_keypair,
            bootnodes,
            listen_on: self.p2p_listen_on,
            blockstore,
            store: store.clone(),
        })?);

        let syncer = Arc::new(Syncer::start(SyncerArgs {
            genesis_hash: self.genesis_hash,
            store: store.clone(),
            p2p: p2p.clone(),
        })?);

        Ok(Node::new(p2p, syncer, store))
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::Path;

    use directories::ProjectDirs;
    use futures::FutureExt;
    use tokio::task::spawn_blocking;
    use tracing::warn;

    use crate::{blockstore::SledBlockstore, store::SledStore, utils};

    use super::*;

    pub(super) async fn default_sled_db(network: &Network) -> Result<sled::Db> {
        let network_id = network.id();
        let mut data_dir = utils::data_dir()
            .ok_or_else(|| StoreError::OpenFailed("Can't find home of current user".into()))?;

        // TODO(02.2024): remove in 3 months or after few releases
        //migrate_from_old_cache_dir(&data_dir).await?;

        data_dir.push(network_id);
        data_dir.push("db");

        let db = spawn_blocking(|| sled::open(data_dir))
            .await
            .map_err(io::Error::from)?
            .map_err(|e| StoreError::OpenFailed(e.to_string()))?;

        Ok(db)
    }

    impl<B, S> NodeBuilder<B, S>
    where
        B: Blockstore,
        S: Store,
    {
        pub fn with_default_blockstore(self) -> NodeBuilder<SledBlockstore, S> {
            NodeBuilder {
                network: self.network,
                genesis_hash: self.genesis_hash,
                p2p_local_keypair: self.p2p_local_keypair,
                p2p_bootnodes: self.p2p_bootnodes,
                p2p_listen_on: self.p2p_listen_on,
                blockstore: Some(StoreKind::InitWithSledDb(Box::new(|db| {
                    async move { Ok(SledBlockstore::new(db).await?) }.boxed()
                }))),
                store: self.store,
            }
        }

        pub fn with_default_store(self) -> NodeBuilder<B, SledStore> {
            NodeBuilder {
                network: self.network,
                genesis_hash: self.genesis_hash,
                p2p_local_keypair: self.p2p_local_keypair,
                p2p_bootnodes: self.p2p_bootnodes,
                p2p_listen_on: self.p2p_listen_on,
                blockstore: self.blockstore,
                store: Some(StoreKind::InitWithSledDb(Box::new(|db| {
                    async move { Ok(SledStore::new(db).await?) }.boxed()
                }))),
            }
        }
    }

    /*
    // TODO(02.2024): remove in 3 months or after few releases
    // Previously we used `.cache/celestia/{network_id}` (or os equivalent) for
    // sled's persistent storage. This function will migrate it to a new lumina
    // data dir. If a new storage is found too, migration will not overwrite it.
    async fn migrate_from_old_cache_dir(data_dir: &Path) -> Result<()> {
        let old_cache_dir = ProjectDirs::from("co", "eiger", "celestia")
            .expect("Must succeed after data_dir is known")
            .cache_dir()
            .to_owned();

        // already migrated or fresh usage
        if !old_cache_dir.exists() {
            return Ok(());
        }

        // we won't migrate old data if user already have a new persistent storage.
        if data_dir.exists() {
            warn!(
                "Found both old and new Lumina storages. {} can be deleted.",
                old_cache_dir.display()
            );
            return Ok(());
        }

        warn!(
            "Migrating Lumina storage to a new location: {} -> {}",
            old_cache_dir.display(),
            data_dir.display()
        );

        // migrate data for each network
        for network in [
            Network::Arabica,
            Network::Mocha,
            Network::Mainnet,
            Network::Private,
        ] {
            let net_id = network.id();
            let old = old_cache_dir.join(net_id);
            let new = data_dir.join(net_id);

            if old.exists() {
                fs::create_dir_all(&new).await?;
                fs::rename(old, new.join("db")).await?;
            }
        }

        if old_cache_dir.read_dir()?.count() > 0 {
            warn!("Old Lumina storage not empty after successful migration.");
            warn!(
                "Inspect and remove it manually: {}",
                old_cache_dir.display()
            );
        }

        Ok(())
    }
    */
}
