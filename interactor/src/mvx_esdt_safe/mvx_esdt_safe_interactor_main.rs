use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk::gateway::SetStateAccount;
use proxies::header_verifier_proxy::HeaderverifierProxy;
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;
use structs::aliases::{OptionalValueTransferDataTuple, PaymentsVec};

use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;
use structs::operation::Operation;

use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;

use common_test_setup::constants::MVX_ESDT_SAFE_CODE_PATH;
use common_test_setup::RegisterTokenArgs;

pub struct MvxEsdtSafeInteract {
    pub interactor: Interactor,
    pub owner_address: Address,
    pub user_address: Address,
    pub state: State,
}
impl CommonInteractorTrait for MvxEsdtSafeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }

    fn wallet_address(&mut self) -> &Address {
        &self.owner_address
    }
}

impl MvxEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        let working_dir = "interactor";
        interactor.set_current_dir_from_workspace(working_dir);
        let owner_address = interactor.register_wallet(test_wallets::mike()).await;
        let user_address = interactor.register_wallet(test_wallets::bob()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1u64).await.unwrap();

        let set_state_response = interactor.set_state_for_saved_accounts().await;
        interactor.generate_blocks(2u64).await.unwrap();
        assert!(set_state_response.is_ok());

        MvxEsdtSafeInteract {
            interactor,
            owner_address,
            user_address,
            state: State::load_state(),
        }
    }

    pub async fn deploy_contracts(
        &mut self,
        sovereign_config: SovereignConfig<StaticApi>,
        esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee_struct: Option<FeeStruct<StaticApi>>,
    ) {
        self.deploy_chain_config(sovereign_config).await;
        self.deploy_header_verifier(self.state.current_chain_config_sc_address().clone())
            .await;
        self.deploy_mvx_esdt_safe(
            self.state.current_header_verifier_address().clone(),
            esdt_safe_config,
        )
        .await;
        self.set_esdt_safe_address_in_header_verifier(
            self.state.current_mvx_esdt_safe_contract_address().clone(),
        )
        .await;
        self.complete_header_verifier_setup_phase().await;
        self.deploy_testing_sc().await;
        self.complete_setup_phase().await;
        self.deploy_fee_market(
            self.state.current_mvx_esdt_safe_contract_address().clone(),
            fee_struct,
        )
        .await;
        self.set_fee_market_address(self.state.current_fee_market_address().to_address())
            .await;
    }

    pub async fn register_operation(
        &mut self,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        self.interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_header_verifier_address())
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                signature,
                hash_of_hashes,
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                operations_hashes,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn complete_setup_phase(&mut self) {
        self.interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .from(&self.owner_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .upgrade()
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn update_configuration(&mut self, new_config: EsdtSafeConfig<StaticApi>) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_configuration(new_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee_market_address(&mut self, fee_market_address: Address) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deposit(
        &mut self,
        to: Address,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payments: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        if let Some(expected_log) = expected_log {
            self.assert_expected_log(logs, expected_log);
        }
    }

    pub async fn execute_operations(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(120_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        if let Some(expected_log) = expected_log {
            self.assert_expected_log(logs, expected_log);
        }
    }

    pub async fn register_token(
        &mut self,
        args: RegisterTokenArgs<'_>,
        egld_amount: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .register_token(
                args.sov_token_id,
                args.token_type,
                args.token_display_name,
                args.token_ticker,
                args.num_decimals,
            )
            .egld(egld_amount)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .pause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn unpause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .unpause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn paused_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .typed(MvxEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn reset_state_chain_sim(&mut self, address_states: Option<Vec<Bech32Address>>) {
        let mut state_vec = vec![
            SetStateAccount::from_address(
                Bech32Address::from(self.owner_address.clone()).to_bech32_string(),
            ),
            SetStateAccount::from_address(
                Bech32Address::from(self.user_address.clone()).to_bech32_string(),
            ),
            SetStateAccount::from_address(
                self.state
                    .current_mvx_esdt_safe_contract_address()
                    .to_bech32_string(),
            ),
            SetStateAccount::from_address(
                self.state
                    .current_header_verifier_address()
                    .to_bech32_string(),
            ),
            SetStateAccount::from_address(
                self.state
                    .current_chain_config_sc_address()
                    .to_bech32_string(),
            ),
        ];

        if let Some(address_states) = address_states {
            for address in address_states {
                state_vec.push(SetStateAccount::from_address(address.to_bech32_string()));
            }
        }
        let response = self.interactor.set_state_overwrite(state_vec).await;
        self.interactor.generate_blocks(2u64).await.unwrap();
        assert!(response.is_ok());
    }
}
