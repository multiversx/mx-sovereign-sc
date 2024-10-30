#![no_std]

multiversx_sc::imports!();

pub type Epoch = u64;
pub const MAX_PERCENTAGE: u64 = 100_000;
pub const APY: u64 = 10_000; //10%
pub const EPOCHS_IN_YEAR: u64 = 365;
pub const UNBOND_PERIOD: u64 = 10;

pub mod delegation_proxy;

#[multiversx_sc::contract]
pub trait DelegationMock {
    #[init]
    fn init(&self) {}

    #[payable("EGLD")]
    #[only_owner]
    #[endpoint(depositEGLD)]
    fn deposit_egld(&self) {
        let payment_amount = self.call_value().egld_value().clone_value();
        self.egld_token_supply()
            .update(|value| *value += &payment_amount);
    }

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(&self) {
        let caller = self.blockchain().get_caller();
        let payment_amount = self.call_value().egld_value().clone_value();
        self.address_deposit(&caller)
            .update(|value| *value += &payment_amount);
        self.egld_token_supply()
            .update(|value| *value += &payment_amount);
    }

    #[endpoint(unDelegate)]
    fn undelegate(&self, egld_to_undelegate: BigUint) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let total_deposit = self.address_deposit(&caller).get();
        require!(
            egld_to_undelegate > 0 && egld_to_undelegate <= total_deposit,
            "Invalid undelegate amount"
        );
        self.address_deposit(&caller)
            .update(|value| *value -= &egld_to_undelegate);
        self.address_undelegate_amount(&caller)
            .update(|value| *value += &egld_to_undelegate);
        self.address_undelegate_epoch(&caller)
            .set(current_epoch + UNBOND_PERIOD);
    }

    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let withdraw_epoch = self.address_undelegate_epoch(&caller).get();
        let withdraw_amount = self.address_undelegate_amount(&caller).get();

        require!(withdraw_amount > 0, "No amount to withdraw");
        require!(
            withdraw_epoch > 0 && current_epoch >= withdraw_epoch,
            "Cannot withdraw yet"
        );

        self.egld_token_supply()
            .update(|value| *value -= &withdraw_amount);
        self.address_undelegate_epoch(&caller).clear();
        self.address_undelegate_amount(&caller).clear();

        self.tx().to(&caller).egld(&withdraw_amount).transfer();
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let last_claim_epoch = self.address_last_claim_epoch(&caller).get();
        let total_deposit = self.address_deposit(&caller).get();

        if current_epoch > last_claim_epoch {
            let rewards = (total_deposit * APY / MAX_PERCENTAGE)
                * (current_epoch - last_claim_epoch)
                / EPOCHS_IN_YEAR;
            if rewards > 0u64 {
                self.tx().to(&caller).egld(&rewards).transfer();
                self.address_last_claim_epoch(&caller).set(current_epoch);
                self.egld_token_supply().update(|value| *value -= &rewards);
            }
        }
    }

    #[storage_mapper("egldTokenSupply")]
    fn egld_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("addressDeposit")]
    fn address_deposit(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("addressLastClaim")]
    fn address_last_claim_epoch(&self, address: &ManagedAddress) -> SingleValueMapper<Epoch>;

    #[storage_mapper("addressUndelegateAmount")]
    fn address_undelegate_amount(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[storage_mapper("addressUndelegateEpoch")]
    fn address_undelegate_epoch(&self, address: &ManagedAddress) -> SingleValueMapper<Epoch>;
}
