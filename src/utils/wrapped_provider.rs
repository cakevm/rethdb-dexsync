use alloy::eips::{BlockNumHash, BlockNumberOrTag};
use alloy_primitives::{BlockHash, BlockNumber, B256};
use reth_chainspec::{ChainInfo, ChainSpec};
use reth_db::DatabaseEnv;
use reth_node_ethereum::EthereumNode;
use reth_node_types::NodeTypesWithDBAdapter;
use reth_provider::{
    BlockHashReader, BlockIdReader, BlockNumReader, DatabaseProviderRO, ProviderError, ProviderFactory, ProviderResult, StateProviderBox,
    StateProviderFactory,
};
use std::sync::Arc;

// Helper wrapper to have StateProviderFactory implemented for exex and db provider
#[derive(Clone)]
pub struct WrappedProviderFactory {
    inner: ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>,
}

impl WrappedProviderFactory {
    pub fn new(inner: ProviderFactory<NodeTypesWithDBAdapter<EthereumNode, Arc<DatabaseEnv>>>) -> Self {
        WrappedProviderFactory { inner }
    }

    pub fn db_ref(&self) -> &Arc<DatabaseEnv> {
        self.inner.db_ref()
    }

    pub fn provider(&self) -> ProviderResult<DatabaseProviderRO<Arc<DatabaseEnv>, ChainSpec>> {
        self.inner.provider()
    }
}

impl BlockIdReader for WrappedProviderFactory {
    fn pending_block_num_hash(&self) -> ProviderResult<Option<BlockNumHash>> {
        todo!()
    }

    fn safe_block_num_hash(&self) -> ProviderResult<Option<BlockNumHash>> {
        todo!()
    }

    fn finalized_block_num_hash(&self) -> ProviderResult<Option<BlockNumHash>> {
        todo!()
    }
}

impl BlockNumReader for WrappedProviderFactory {
    fn chain_info(&self) -> ProviderResult<ChainInfo> {
        todo!()
    }

    fn best_block_number(&self) -> ProviderResult<BlockNumber> {
        todo!()
    }

    fn last_block_number(&self) -> ProviderResult<BlockNumber> {
        todo!()
    }

    fn block_number(&self, _hash: B256) -> ProviderResult<Option<BlockNumber>> {
        todo!()
    }
}

impl BlockHashReader for WrappedProviderFactory {
    fn block_hash(&self, _number: BlockNumber) -> ProviderResult<Option<B256>> {
        todo!()
    }

    fn canonical_hashes_range(&self, _start: BlockNumber, _end: BlockNumber) -> ProviderResult<Vec<B256>> {
        todo!()
    }
}

impl StateProviderFactory for WrappedProviderFactory {
    fn latest(&self) -> ProviderResult<StateProviderBox> {
        self.inner.latest()
    }

    fn state_by_block_number_or_tag(&self, number_or_tag: BlockNumberOrTag) -> ProviderResult<StateProviderBox> {
        match number_or_tag {
            BlockNumberOrTag::Number(number) => self.history_by_block_number(number),
            BlockNumberOrTag::Earliest => self.history_by_block_number(0),
            BlockNumberOrTag::Latest => self.latest(),
            BlockNumberOrTag::Pending => self.pending(),
            BlockNumberOrTag::Safe => Err(ProviderError::UnsupportedProvider),
            BlockNumberOrTag::Finalized => Err(ProviderError::UnsupportedProvider),
        }
    }

    fn history_by_block_number(&self, block_number: BlockNumber) -> ProviderResult<StateProviderBox> {
        self.inner.history_by_block_number(block_number)
    }

    fn history_by_block_hash(&self, block: BlockHash) -> ProviderResult<StateProviderBox> {
        self.inner.history_by_block_hash(block)
    }

    fn state_by_block_hash(&self, block: BlockHash) -> ProviderResult<StateProviderBox> {
        self.inner.history_by_block_hash(block)
    }

    fn pending(&self) -> ProviderResult<StateProviderBox> {
        todo!()
    }

    fn pending_state_by_hash(&self, _block_hash: B256) -> ProviderResult<Option<StateProviderBox>> {
        todo!()
    }
}
