# AZ Smart Contract Hub

## Getting Started
### Prerequisites

* [Cargo](https://doc.rust-lang.org/cargo/)
* [Rust](https://www.rust-lang.org/)
* [ink!](https://use.ink/)
* [Cargo Contract v4.0.0-alpha](https://github.com/paritytech/cargo-contract)
```zsh
cargo install --git https://github.com/paritytech/cargo-contract cargo-contract --force
```

### Checking code

```zsh
cargo checkmate
cargo sort
```

## Testing

### Run unit tests

```sh
cargo test
```

### Run integration tests

```sh
export CONTRACTS_NODE="/Users/myname/.cargo/bin/substrate-contracts-node"
cargo test --features e2e-tests
```

#### Why not ink-wrapper?

Could not get [ink-wrapper](https://docs.alephzero.org/aleph-zero/build/writing-e2e-tests-with-ink-wrapper) to work:

```
thread 'tests::it_works' panicked at 'Rpc(ClientError(Transport(Error when opening the TCP socket: Connection refused (os error 61))))'
```

Note that I am using MacOS.

## Deployment

1. Build contract:
```sh
# You may need to run
# chmod +x build.sh f
./build.sh
```
2. If setting up locally, start a local development chain. 
```sh
substrate-contracts-node --dev
```
3. Upload, initialise and interact with contract at [Contracts UI](https://contracts-ui.substrate.io/).

## References

- https://substrate.stackexchange.com/questions/7881/error-loading-of-original-wasm-failed
- https://substrate.stackexchange.com/questions/8625/trying-to-implement-u256-in-rust-ink-4-0
- https://github.com/swanky-dapps/manic-minter
- https://docs.astar.network/docs/build/wasm/from-zero-to-ink-hero/manic-minter/manic-contract/
- https://docs.rs/primitive-types/latest/primitive_types/struct.U256.html#impl-ToString-for-U256
- https://github.com/btn-group/profit-distributor-b/blob/master/src/contract.rs
- https://learn.brushfam.io/docs/OpenBrush/smart-contracts/example/impls
- https://docs.azero.id/developers/deployments#metadata
- https://github.com/azero-id/resolver
- https://docs.alephzero.org/aleph-zero/build/writing-e2e-tests-with-ink-wrapper
- https://github.com/paritytech/ink-examples/tree/61f69a77b3e32fe18c1f144a2863d25471778bee/multi-contract-caller
- https://docs.rs/ink_e2e/4.3.0/ink_e2e/index.html
