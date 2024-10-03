# RethDB DexSync
This crate provides a way to read Uniswap v2/v3 pool states direct from the Reth database. This will speed up the initial sync process and reduce the load on the Ethereum node.

## Features
- Reading Uniswap v2 pairs/reserves
- Reading Uniswap v3 pools positions and slot0

## Usage
The `RETH_DB_PATH` without /db/ directory. For example, if the path is `/home/user/reth/db/`, then the `RETH_DB_PATH` should be `/home/user/reth`.
```
RETH_DB_PATH=<your_reth_db_path>
cargo run --release --example univ2_pairs
cargo run --release --example univ3_pools
```

## Acknowledgements
Many thanks to the team of [reth](https://github.com/paradigmxyz/reth).

## License
This project is dual-licensed under the [MIT](./LICENSE-MIT) and [Apache-2.0](./LICENSE-APACHE) licenses.