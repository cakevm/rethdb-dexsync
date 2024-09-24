# Reading Uniswap v2/v3 pairs/pools using reth direct db access

## Features
- Reading Uniswap v2 pairs using reth direct db access
- Reading Uniswap v3 pools addresses by parsing event logs

## Usage
The `RETH_DB_PATH` without /db/ directory. For example, if the path is `/home/user/reth/db/`, then the `RETH_DB_PATH` should be `/home/user/reth`.
```
RETH_DB_PATH=<your_reth_db_path>
cargo run --release --example univ2_pairs
cargo run --release --example univ3_pools
```

## Acknowledgements
This crate is based on the [reth db-access example](https://github.com/paradigmxyz/reth/tree/main/examples/db-access).

## License
This project is licensed under the [Apache 2.0](./LICENSE).