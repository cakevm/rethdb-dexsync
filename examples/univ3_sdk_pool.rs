use alloy_primitives::aliases::I24;
use alloy_primitives::{address, Address};
use reth_storage_api::StateProviderFactory;
use rethdb_dexsync::univ3::UniswapV3Pool;
use rethdb_dexsync::utils::{init_db_read_only_from_env, WrappedProviderFactory};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use uniswap_sdk_core::prelude::Token;
use uniswap_sdk_core::prelude::{BaseCurrency, CurrencyAmount, Ether};
use uniswap_sdk_core::token;
use uniswap_v3_sdk::constants::FeeAmount;
use uniswap_v3_sdk::entities::Tick;
use uniswap_v3_sdk::error::Error;
use uniswap_v3_sdk::prelude::{Pool, TickDataProvider};

#[derive(Clone)]
struct RethDbTickDataProvider {
    factory: WrappedProviderFactory,
    pool_address: Address,
    ticks: RefCell<HashMap<I24, Tick<I24>>>,
}
impl RethDbTickDataProvider {
    fn new(factory: WrappedProviderFactory, pool_address: Address, ticks: RefCell<HashMap<I24, Tick<I24>>>) -> Self {
        Self { factory, pool_address, ticks }
    }
}

impl TickDataProvider for RethDbTickDataProvider {
    type Index = I24;

    fn get_tick(&self, tick_id: Self::Index) -> Result<&Tick<Self::Index>, Error> {
        let Ok(state_provider) = self.factory.latest() else { return Err(Error::InvalidTick(tick_id)) };
        let Ok(tick_info_optional) = UniswapV3Pool::read_tick(state_provider, self.pool_address, tick_id) else {
            return Err(Error::InvalidTick(tick_id));
        };
        let Some(tick_info) = tick_info_optional else { return Err(Error::InvalidTick(tick_id)) };

        let tick = Tick::new(tick_id, tick_info.liquidity_gross.to(), i128::try_from(tick_info.liquidity_net).unwrap());
        self.ticks.borrow_mut().insert(tick_id, tick);

        let ticks_ref = self.ticks.borrow();
        match ticks_ref.get(&tick_id) {
            Some(v) => Ok(v),
            None => Err(Error::InvalidTick(tick_id)),
        }
    }

    fn next_initialized_tick_within_one_word(
        &self,
        tick: Self::Index,
        lte: bool,
        tick_spacing: Self::Index,
    ) -> Result<(Self::Index, bool), Error> {
        Ok((I24::ZERO, false))
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let factory = init_db_read_only_from_env()?;

    let pool_address = address!("88e6a0c2ddd26feeb64f039a2c41296fcb3f5640");
    let slot0 = UniswapV3Pool::read_slot0(factory.latest()?, pool_address)?.unwrap();
    let liquidity: u128 = UniswapV3Pool::read_liquidity(factory.latest()?, pool_address)?.to();

    let ticks = RefCell::new(HashMap::new());
    let tick_provider = RethDbTickDataProvider::new(factory, pool_address, ticks.clone());

    let token0 = token!(1, "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", 6, "USDC", "USD Coin");
    let token1 = Ether::on_chain(1).wrapped().clone();
    // USDC/WETH 500: 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640
    let pool = Pool::new_with_tick_data_provider(token0.clone(), token1, FeeAmount::LOW, slot0.sqrt_price_x96, liquidity, tick_provider)?;

    let (amount_out, _) = pool.get_output_amount(&CurrencyAmount::from_raw_amount(token0, 100_000_000).unwrap(), None)?;

    println!("amount_out: {:?}", amount_out);

    Ok(())
}
