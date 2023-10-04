# AZ Smart Contract Hub

A smart contract that allows the decentralised sharing of smart contract info:
```
pub struct SmartContract {
    id: u32,
    smart_contract_address: AccountId,
    chain: u8,
    caller: AccountId,
    enabled: bool,
    azero_id: String,
    abi_url: String,
    contract_url: Option<String>,
    wasm_url: Option<String>,
    audit_url: Option<String>,
    group_id: Option<u32>,
    project_name: Option<String>,
    project_website: Option<String>,
    github: Option<String>,
}
```
A use case is the easing of development and auditing by allowing users to easily access abis to use in Substrate Contracts UI, smart contracts and front end dapp development.

### Rules & notes

**Creating a smart contract record**:
* Chain will be left as type u8 to account for new testnets that may appear. Production will be 0 and Testnet will be 1.
* Caller must own an AZERO.ID and associate it with a record.
* A link to the abi_url (metadata.json) must be provided. In an ideal world, the link would be directed at the location of the smart contract's metadata.json on a CDN.
* If a group_id is provided, the caller must be a member of that group.
* The smart contract record is enabled by default.
```
fn create(
    &mut self,
    smart_contract_address: AccountId,
    chain: u8,
    azero_id: String,
    abi_url: String,
    contract_url: Option<String>,
    wasm_url: Option<String>,
    audit_url: Option<String>,
    group_id: Option<u32>,
    project_name: Option<String>,
    project_website: Option<String>,
    github: Option<String>,
) -> Result<SmartContract> {
```
**Updating a smart contract record**:
* Can only update own smart contract records.
* Caller must own an AZERO.ID and associate it with a record. This means that if a user relinquishes the original azero_id, they must associate a new one on update.
* If a group_id is provided, the caller must be a member of that group.
* Some fields are unable to be updated for security purposes. If some fields are incorrect and are unable to be changed, the user should disable the record and create a new one.
```
fn update(
    &mut self,
    id: u32,
    enabled: bool,
    azero_id: String,
    group_id: Option<u32>,
    audit_url: Option<String>,
    project_name: Option<String>,
    project_website: Option<String>,
    github: Option<String>,
) -> Result<SmartContract> {
```

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

- https://github.com/btn-group/az_groups
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
