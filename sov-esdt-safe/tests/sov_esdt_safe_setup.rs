use multiversx_sc::{
    imports::OptionalValue,
    types::{EsdtLocalRole, ManagedAddress, ManagedVec, TestSCAddress, TokenIdentifier},
};

use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, scenario_model::Log, ReturnsHandledOrError, ReturnsLogs,
    ScenarioTxRun, ScenarioTxWhitebox,
};

use common_blackbox_setup::{
    BaseSetup, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, OWNER_ADDRESS, TEST_TOKEN_ONE,
    TEST_TOKEN_TWO,
};
use proxies::sov_esdt_safe_proxy::SovEsdtSafeProxy;
use sov_esdt_safe::SovEsdtSafe;
use structs::{
    aliases::{OptionalValueTransferDataTuple, PaymentsVec},
    configs::EsdtSafeConfig,
};

pub const SOV_ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("output/to-sovereign.mxsc.json");

pub struct SovEsdtSafeTestState {
    pub common_setup: BaseSetup,
}

impl SovEsdtSafeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut common_setup = BaseSetup::new();
        common_setup
            .world
            .register_contract(SOV_ESDT_SAFE_CODE_PATH, sov_esdt_safe::ContractBuilder);

        Self { common_setup }
    }

    pub fn deploy_contract(
        &mut self,
        fee_market_address: TestSCAddress,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .init(fee_market_address, opt_config)
            .code(SOV_ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn deploy_contract_with_roles(&mut self) -> &mut Self {
        self.common_setup
            .world
            .account(ESDT_SAFE_ADDRESS)
            .nonce(1)
            .code(SOV_ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS)
            .esdt_roles(
                TokenIdentifier::from(TEST_TOKEN_ONE),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                ],
            )
            .esdt_roles(
                TokenIdentifier::from(TEST_TOKEN_TWO),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                ],
            )
            .esdt_roles(
                TokenIdentifier::from(FEE_TOKEN),
                vec![
                    EsdtLocalRole::Burn.name().to_string(),
                    EsdtLocalRole::NftBurn.name().to_string(),
                ],
            );

        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(sov_esdt_safe::contract_obj, |sc| {
                let config = EsdtSafeConfig::new(
                    None,
                    ManagedVec::new(),
                    ManagedVec::new(),
                    50_000_000,
                    ManagedVec::new(),
                );

                sc.init(
                    FEE_MARKET_ADDRESS.to_managed_address(),
                    OptionalValue::Some(config),
                );
            });

        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn deposit(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        opt_payment: Option<PaymentsVec<StaticApi>>,
        expected_error_message: Option<&str>,
    ) {
        let tx = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .deposit(to, opt_transfer_data);

        let response = if let Some(payment) = opt_payment {
            tx.payment(payment)
                .returns(ReturnsHandledOrError::new())
                .run()
        } else {
            tx.returns(ReturnsHandledOrError::new()).run()
        };

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn set_fee_market_address(&mut self, fee_market_address: TestSCAddress) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .run();
    }

    pub fn deposit_with_logs(
        &mut self,
        to: ManagedAddress<StaticApi>,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payment: PaymentsVec<StaticApi>,
    ) -> Vec<Log> {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payment)
            .returns(ReturnsLogs)
            .run()
    }
}
