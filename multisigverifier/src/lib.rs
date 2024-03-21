#![no_std]

use bls_signature::BlsSignature;

mod esdt_safe_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait EsdtSafeProxy {
        #[view(registerPendingOperations)]
        fn register_pending_operations(&self, pending_operations: MultiValueEncoded<ManagedBuffer>);
    }
}

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Multisigverifier: bls_signature::BlsSignatureModule {
    #[init]
    fn init(
        &self,
        esdt_safe_address: ManagedAddress,
        register_ops_whitelist: MultiValueEncoded<ManagedAddress>,
    ) {
        self.esdt_safe_address().set(esdt_safe_address);

        for whitelisted_address in register_ops_whitelist {
            self.register_ops_whitelist().insert(whitelisted_address);
        }
    }

    #[endpoint]
    fn upgrade(&self) {}

    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        signature: BlsSignature<Self::Api>,
        bridge_operations_hash: ManagedBuffer,
        operations_hashes: MultiValueEncoded<ManagedBuffer>,
    ) {
        let caller = self.blockchain().get_caller();

        require!(
            self.register_ops_whitelist().contains(&caller),
            "User has no permission to register an operation"
        );

        require!(
            !self
                .hash_of_hashes_history()
                .contains(&bridge_operations_hash),
            "OutGoingTxHashes is already registered"
        );

        let is_bls_valid = self.verify_bls(&signature, &bridge_operations_hash);

        require!(is_bls_valid, "BLS signature is not valid");

        self.calculate_and_check_transfers_hashes(
            &bridge_operations_hash,
            operations_hashes.clone(),
        );

        self.hash_of_hashes_history()
            .insert(bridge_operations_hash.clone());

        let _ = self
            .esdt_safe_proxy(self.esdt_safe_address().get())
            .register_pending_operations(operations_hashes);
    }

    fn calculate_and_check_transfers_hashes(
        &self,
        transfers_hash: &ManagedBuffer,
        transfers_data: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut transfers_hashes = ManagedBuffer::new();

        for transfer in transfers_data {
            transfers_hashes.append(&transfer);
        }

        let hash_of_hashes_sha256 = self.crypto().sha256(&transfers_hashes);
        let hash_of_hashes = hash_of_hashes_sha256.as_managed_buffer();

        require!(
            transfers_hash.eq(hash_of_hashes),
            "Hash of all operations doesn't match the hash of transfer data"
        );
    }

    fn verify_bls(
        &self,
        _signature: &BlsSignature<Self::Api>,
        _bridge_operations_hash: &ManagedBuffer,
    ) -> bool {
        true
    }

    fn is_signature_count_valid(&self, pub_keys_count: usize) -> bool {
        let total_bls_pub_keys = self.bls_pub_keys().len();
        let minimum_signatures = 2 * total_bls_pub_keys / 3;

        pub_keys_count > minimum_signatures
    }

    #[proxy]
    fn esdt_safe_proxy(&self, sc_address: ManagedAddress) -> esdt_safe_proxy::Proxy<Self::Api>;

    #[storage_mapper("bls_pub_keys")]
    fn bls_pub_keys(&self) -> SetMapper<ManagedBuffer>;

    #[storage_mapper("hash_of_hashes_history")]
    fn hash_of_hashes_history(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("registerOpsWhitelist")]
    fn register_ops_whitelist(&self) -> UnorderedSetMapper<ManagedAddress>;
}
