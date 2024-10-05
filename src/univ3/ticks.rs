use alloy::primitives::aliases::I24;
use alloy::primitives::{b256, keccak256, B256};
use alloy_sol_types::SolValue;
use lazy_static::lazy_static;
use std::collections::HashMap;

const MIN_TICK: i32 = -887272;
const MAX_TICK: i32 = -MIN_TICK;

const TICKS_SLOT: B256 = b256!("0000000000000000000000000000000000000000000000000000000000000005");

macro_rules! fill_slot_keys_ticks_map {
    ($slot:expr, $tick_spacing:expr) => {{
        let mut map = HashMap::new();
        for tick in (0..=MAX_TICK).step_by($tick_spacing) {
            let storage_key: B256 = keccak256((I24::try_from(tick).unwrap(), TICKS_SLOT).abi_encode());
            map.insert(storage_key, tick);
            let storage_key: B256 = keccak256((I24::try_from(-tick).unwrap(), TICKS_SLOT).abi_encode());
            map.insert(storage_key, -tick);
        }

        map
    }};
}

lazy_static! {
    pub static ref SLOT_KEYS_TO_TICKS: HashMap<B256, i32> = fill_slot_keys_ticks_map!(TICKS_SLOT, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn num_ticks(tick_spacing: i32) -> u32 {
        let min_tick = (MIN_TICK / tick_spacing) * tick_spacing;
        let max_tick = (MAX_TICK / tick_spacing) * tick_spacing;
        let num_ticks = ((max_tick - min_tick) / tick_spacing) as u32 + 1;
        num_ticks
    }

    #[test]
    fn test_fill_ticks() {
        // fee => tick, 10000 => 200, 3000 => 60, 500 => 10, 100 => 1

        let tick_spacing: usize = 1;
        let num_ticks = num_ticks(tick_spacing as i32);

        let map: HashMap<B256, i32> = fill_slot_keys_ticks_map!(TICKS_SLOT, tick_spacing);
        assert_eq!(*map.get(&b256!("87361ea236b1c1a4b101e72bd6c912613e5b68034f169d3f702e04d520b95e40")).unwrap(), 1740);
        assert_eq!(*map.get(&b256!("f0d9527cb167031b75168a62435c989786c3c280f4295693c5c862fd764aad88")).unwrap(), 3180);
        assert_eq!(*map.get(&b256!("ce86dc00f32e6e2f4f3dc9f672a068ead5635d305cd72b916224201b344d2c6e")).unwrap(), -345420);

        assert_eq!(num_ticks, map.len() as u32);
    }
}
