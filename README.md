# Sovereign Smart Contracts

## Abstract
Institutions which want to use blockchain technology, often want to run their own specifications (contracts, language, VM, nodes), and they often want to run privately.

With Sovereign Shards we want to give an SDK for all builders to run their own highly efficient chains, but which is seamlessly connected and integrated into the global markets of MultiversX. Give developers freedom, give users the power to enjoy all dApps seamlessly, make it feel like they are on a single blockchain. One wallet to interact with all. Composability and interoperability built into the design and architecture of the system. New paradigms to leverage existing infrastructure and enhance the user experience. Usability of all chains simplified by systems like relayed transactions and paymaster smart contracts.

# Multisigverifier Contract

## Endpoints
### init
```rust
    #[init]
    fn init(&self, bls_pub_keys: MultiValueEncoded<ManagedBuffer>);
```
    The init function is called when deploying/upgrading a smart contract
