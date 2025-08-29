use error_messages::{
    ADDITIONAL_STAKE_NOT_REQUIRED, ADDITIONAL_STAKE_ZERO_VALUE, INVALID_ADDITIONAL_STAKE,
    INVALID_BLS_KEY_FOR_CALLER, INVALID_EGLD_STAKE, INVALID_MIN_MAX_VALIDATOR_NUMBERS,
    INVALID_TOKEN_ID,
};
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
use structs::{configs::SovereignConfig, ValidatorInfo};

use crate::storage;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait ChainConfigUtilsModule: storage::ChainConfigStorageModule {
    // What should be the maximum number of validators ?
    fn is_new_config_valid(&self, config: &SovereignConfig<Self::Api>) -> Option<&str> {
        if let Some(additional_stake) = config.opt_additional_stake_required.clone() {
            for stake in additional_stake {
                if !stake.token_identifier.is_valid_esdt_identifier() {
                    return Some(INVALID_TOKEN_ID);
                }
                if stake.amount <= 0 {
                    return Some(ADDITIONAL_STAKE_ZERO_VALUE);
                }
            }
        }

        if config.min_validators <= config.max_validators {
            None
        } else {
            Some(INVALID_MIN_MAX_VALIDATOR_NUMBERS)
        }
    }

    fn refund_stake(
        &self,
        caller: &ManagedAddress<Self::Api>,
        validator_info: &ValidatorInfo<Self::Api>,
    ) {
        let stake = self.get_total_stake(validator_info);
        if stake.is_empty() {
            return;
        }

        self.tx()
            .to(caller)
            .payment(stake)
            .transfer_execute();
    }

    fn get_total_stake(
        &self,
        validator_info: &ValidatorInfo<Self::Api>,
    ) -> MultiEgldOrEsdtPayment<Self::Api> {
        let mut total_stake = MultiEgldOrEsdtPayment::new();
        if validator_info.egld_stake > 0 {
            total_stake.push(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::from(ManagedBuffer::from(EGLD_000000_TOKEN_IDENTIFIER)),
                0,
                validator_info.egld_stake.clone(),
            ));
        }

        if let Some(additional_stake) = &validator_info.token_stake {
            for stake in additional_stake {
                total_stake.push(stake.clone().into());
            }
        }

        total_stake
    }

    fn validate_stake(
        &self,
    ) -> (
        BigUint<Self::Api>,
        Option<ManagedVec<EsdtTokenPayment<Self::Api>>>,
    ) {
        let sovereign_config = self.sovereign_config().get();

        let (egld_amount, esdt_payments) = self.split_payments();

        require!(
            egld_amount == sovereign_config.min_stake,
            INVALID_EGLD_STAKE
        );

        if let Some(additional) = &sovereign_config.opt_additional_stake_required {
            let valid = additional.iter().all(|s| {
                esdt_payments
                    .iter()
                    .any(|p| p.token_identifier == s.token_identifier && p.amount == s.amount)
            });
            require!(valid, INVALID_ADDITIONAL_STAKE);
        } else {
            require!(esdt_payments.is_empty(), ADDITIONAL_STAKE_NOT_REQUIRED);
        }

        (egld_amount, Some(esdt_payments))
    }

    fn split_payments(&self) -> (BigUint, ManagedVec<EsdtTokenPayment<Self::Api>>) {
        let mut egld_amount = BigUint::zero();
        let mut esdt_payments = ManagedVec::new();

        for payment in self.call_value().all_transfers().clone_value().into_iter() {
            if payment.token_identifier.is_egld() {
                egld_amount += payment.amount.clone();
            } else {
                esdt_payments.push(payment.unwrap_esdt());
            }
        }

        (egld_amount, esdt_payments)
    }

    fn require_caller_has_bls_key(
        &self,
        caller: &ManagedAddress<Self::Api>,
        validator_info: &ValidatorInfo<Self::Api>,
    ) {
        require!(
            validator_info.address == *caller,
            INVALID_BLS_KEY_FOR_CALLER
        );
    }
}
