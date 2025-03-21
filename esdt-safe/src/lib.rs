#![no_std]

use transaction::GasLimit;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod esdt_safe_proxy;
pub mod from_sovereign;
pub mod to_sovereign;

const MAX_USER_TX_GAS_LIMIT: GasLimit = 60_000_000;

#[multiversx_sc::contract]
pub trait EsdtSafe:
    to_sovereign::create_tx::CreateTxModule
    + to_sovereign::events::EventsModule
    + bls_signature::BlsSignatureModule
    + from_sovereign::events::EventsModule
    + from_sovereign::token_mapping::TokenMappingModule
    + from_sovereign::transfer_tokens::TransferTokensModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + setup_phase::SetupPhaseModule
    + token_whitelist::TokenWhitelistModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self, is_sovereign_chain: bool, opt_native_token: OptionalValue<ManagedBuffer>) {
        self.is_sovereign_chain().set(is_sovereign_chain);
        self.max_user_tx_gas_limit().set(MAX_USER_TX_GAS_LIMIT);

        if let OptionalValue::Some(native_token) = opt_native_token {
            require!(
                TokenIdentifier::from(native_token.clone()).is_valid_esdt_identifier(),
                "Native token is not a valid token identifier"
            );
            self.native_token().set(native_token);
        }

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);

        self.fee_market_address().set(fee_market_address);
    }

    #[only_owner]
    #[endpoint(setHeaderVerifierAddress)]
    fn set_header_verifier_address(&self, header_verifier_address: ManagedAddress) {
        self.require_sc_address(&header_verifier_address);

        self.header_verifier_address().set(&header_verifier_address);
    }

    #[only_owner]
    #[endpoint(setMaxTxGasLimit)]
    fn set_max_user_tx_gas_limit(&self, max_user_tx_gas_limit: GasLimit) {
        self.max_user_tx_gas_limit().set(max_user_tx_gas_limit);
    }

    #[only_owner]
    #[endpoint(setBannedEndpoint)]
    fn set_banned_endpoint(&self, endpoint_name: ManagedBuffer) {
        self.banned_endpoint_names().insert(endpoint_name);
    }
}
