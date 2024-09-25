use crate::univ3_factory::UniswapV3Factory::PoolCreated;
use alloy::primitives::{address, b256, Address, B256};
use alloy_rpc_types::{BlockNumHash, Filter, FilteredParams};
use alloy_sol_types::sol;
use reth_primitives::BlockHashOrNumber;
use reth_provider::{BlockReader, ProviderError};
use reth_rpc_eth_types::logs_utils::append_matching_block_logs;
use std::iter::StepBy;
use std::ops::RangeInclusive;

sol! (
    contract UniswapV3Factory {
        event PoolCreated(address indexed token0, address indexed token1, uint24 fee, int24 tick);
    }
);

const POOL_CREATED: B256 = b256!("783cca1c0412dd0d695e784568c96da2e9c22ff989357a2e8b1d9b2b4e6b7118");

pub fn read_univ3_pools<T: BlockReader>(provider: T) -> eyre::Result<Vec<Address>> {
    let from_block = 12369621u64; // deployment of univ3 factory
    let to_block = provider.last_block_number()?; // current block number
    let max_headers_range = 10000u64;

    let univ3_factory = address!("1F98431c8aD98523631AE4a59f267346ea31F984");

    let filter = Filter::default().address(univ3_factory).event_signature(vec![POOL_CREATED]);
    let filter_params = FilteredParams::new(Some(filter.clone()));

    let address_filter = FilteredParams::address_filter(&filter.address);
    let topics_filter = FilteredParams::topics_filter(&filter.topics);

    let mut all_logs = vec![];

    println!("All blocks: {} -> {}", from_block, to_block);
    for (from, to) in BlockRangeInclusiveIter::new(from_block..=to_block, max_headers_range) {
        let headers = provider.headers_range(from..=to)?;
        println!("from: {}, to: {}, len: {}", from, to, headers.len());

        for (idx, header) in headers.iter().enumerate() {
            // only if filter matches
            if FilteredParams::matches_address(header.logs_bloom, &address_filter)
                && FilteredParams::matches_topics(header.logs_bloom, &topics_filter)
            {
                // these are consecutive headers, so we can use the parent hash of the next
                // block to get the current header's hash
                let block_hash = match headers.get(idx + 1) {
                    Some(parent) => parent.parent_hash,
                    None => provider.block_hash(header.number)?.ok_or(ProviderError::HeaderNotFound(header.number.into()))?,
                };

                if let Some(receipts) = provider.receipts_by_block(BlockHashOrNumber::from(block_hash))? {
                    append_matching_block_logs(
                        &mut all_logs,
                        &provider,
                        &filter_params,
                        BlockNumHash::new(header.number, block_hash),
                        &receipts,
                        false,
                        header.timestamp,
                    )?;
                }
            }
        }
    }
    let mut pools = vec![];
    for log in all_logs {
        let decoded = log.log_decode::<PoolCreated>()?;
        pools.push(decoded.inner.address);
    }
    Ok(pools)
}

// CODE FROM reth: eth/filter.rs
/// An iterator that yields _inclusive_ block ranges of a given step size
#[derive(Debug)]
struct BlockRangeInclusiveIter {
    iter: StepBy<RangeInclusive<u64>>,
    step: u64,
    end: u64,
}

impl BlockRangeInclusiveIter {
    fn new(range: RangeInclusive<u64>, step: u64) -> Self {
        Self { end: *range.end(), iter: range.step_by(step as usize + 1), step }
    }
}

impl Iterator for BlockRangeInclusiveIter {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.iter.next()?;
        let end = (start + self.step).min(self.end);
        if start > end {
            return None;
        }
        Some((start, end))
    }
}
