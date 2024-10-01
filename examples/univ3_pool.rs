use alloy::primitives::address;
use reth_direct_db_uniswap_storage::univ3::read_slot0;
use reth_direct_db_uniswap_storage::utils::init_db_from_env;

fn main() -> eyre::Result<()> {
    let factory = init_db_from_env()?;

    // Read all pools from UniswapV3Factory
    let slot0 = read_slot0(&factory.latest()?, address!("7bea39867e4169dbe237d55c8242a8f2fcdcc387"))?;

    println!("Slot0: {:#?}", slot0);

    Ok(())
}
