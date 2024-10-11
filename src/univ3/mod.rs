mod ticks;
mod univ3_pool;
mod univ3_position;

use alloy_primitives::{address, Address};
pub use univ3_pool::{read_slot0, Univ3Pool, Univ3Slot0};
pub use univ3_position::{read_univ3_position_pools, UniV3PositionManager};

pub const UNI_V3_FACTORY: Address = address!("1F98431c8aD98523631AE4a59f267346ea31F984");
pub const UNI_V3_POSITION_MANAGER: Address = address!("c36442b4a4522e871399cd717abdd847ab11fe88");
