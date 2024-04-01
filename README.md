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
The init function is called when deploying/upgrading a smart contract. The __bls_pub_keys__ are inserted inside the contract's storage to know the public keys of the validators.

### register_bridge_operations
```rust
    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        signature: BlsSignature<Self::Api>,
        bridge_operations_hash: ManagedBuffer,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ); 
```
- __signature__: the BLS multisignature that should prove the authenticity of the operations that are being registered
- __bridge_operations_hash__: the hash of all operations that will be registered, it is used as a key for further reference and checks, also known as __hash_of_hashes__
- __operations_hashes__: an array of hashes, each being the corresponding hash of one operation

Registering one or more operations is the first step to be able to execute them on the bridge smart contract after that. This endpoint will verify some conditions in order to successfully insert the operations in the storage:
1. The __bridge_operations_hash__ can't be registered twice
2. The BLS Multisignature has to be valid
3. The __bridge_operations_hash__ and hash of __operations_hashes__ have to be equal 

After the conditions are checked and valid, the contract's storage will be updated with the __operations_hashes__ as __pending_hashes__ and the __bridge_operations_hash__ in the __hash_of_hashes_history__ mapper.

### set_esdt_safe_address
```rust
    #[only_owner]
    #[endpoint(setEsdtSafeAddress)]
    fn set_esdt_safe_address(&self, esdt_safe_address: ManagedAddress)
```
To be able to do calls with the Esdt Safe bridge contract, the Multisigverifier contract has to know the address of the bridge.
