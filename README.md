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
* There is a fee to create which is sent to the admin.
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

A combination of unit tests and integration tests are present. Integration tests were written mainly to test interactions with the AZ Groups smart contract. Integration tests with the AZERO.ID router could not be written as that contract is private. Different scenarios that could happen while interacting with the AZERO.ID router are simulated with mock addresses and domains that can't exist in production.

### Run unit tests

```sh
cargo test
```

### Run integration tests

```sh
# export CONTRACTS_NODE="/Users/myname/.cargo/bin/substrate-contracts-node"
cargo test --features e2e-tests
```

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

- [AZ Groups Github](https://github.com/btn-group/az_groups)
- [AZERO.ID Metadata](https://docs.azero.id/developers/deployments#metadata)
- [AZERO.ID Resolver](https://github.com/azero-id/resolver)
- [INK Multi-Contract-Caller Example](https://github.com/paritytech/ink-examples/tree/61f69a77b3e32fe18c1f144a2863d25471778bee/multi-contract-caller)
- [ink_e2e Docs](https://docs.rs/ink_e2e/4.3.0/ink_e2e/index.html)
