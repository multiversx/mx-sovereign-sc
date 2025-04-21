#![no_std]
pub mod constants;
use constants::{
    CHAIN_CONFIG_ADDRESS, CHAIN_CONFIG_CODE_PATH, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS,
    FEE_MARKET_CODE_PATH, HEADER_VERIFIER_ADDRESS, HEADER_VERIFIER_CODE_PATH, OWNER_ADDRESS,
    TESTING_SC_ADDRESS, TESTING_SC_CODE_PATH,
};
use cross_chain::storage::CrossChainStorage;
use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        BigUint, ContractBase, EgldOrEsdtTokenIdentifier, EsdtTokenType, ManagedAddress,
        ManagedBuffer, MultiValue3, TestAddress, TestTokenIdentifier, TokenIdentifier, Vec,
    },
    multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::{Log, TxResponseStatus},
    DebugApi, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};
use mvx_esdt_safe::bridging_mechanism::BridgingMechanism;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    header_verifier_proxy::HeaderverifierProxy,
    testing_sc_proxy::TestingScProxy,
};
use structs::configs::SovereignConfig;

pub struct RegisterTokenArgs<'a> {
    pub sov_token_id: TokenIdentifier<StaticApi>,
    pub token_type: EsdtTokenType,
    pub token_display_name: &'a str,
    pub token_ticker: &'a str,
    pub num_decimals: usize,
}

pub struct BaseSetup {
    pub world: ScenarioWorld,
}

pub struct AccountSetup<'a> {
    pub address: TestAddress<'a>,
    pub esdt_balances: Option<Vec<(TestTokenIdentifier<'a>, BigUint<StaticApi>)>>,
    pub egld_balance: Option<BigUint<StaticApi>>,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);

    blockchain
}

impl BaseSetup {
    #[allow(clippy::new_without_default)]
    pub fn new(account_setups: Vec<AccountSetup>) -> Self {
        let mut world = world();

        for acc in account_setups {
            let mut acc_builder = world.account(acc.address).nonce(1);

            if let Some(esdt_balances) = acc.esdt_balances {
                for (token_id, amount) in esdt_balances {
                    acc_builder = acc_builder.esdt_balance(token_id, amount);
                }
            }

            if let Some(balance) = acc.egld_balance {
                acc_builder.balance(balance);
            }
        }

        Self { world }
    }

    pub fn deploy_fee_market(&mut self, fee: Option<FeeStruct<StaticApi>>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FeeMarketProxy)
            .init(ESDT_SAFE_ADDRESS, fee)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    pub fn deploy_testing_sc(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .new_address(TESTING_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_header_verifier(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init()
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    pub fn deploy_chain_config(&mut self, config: SovereignConfig<StaticApi>) -> &mut Self {
        // let mut additional_stake_as_tuple = MultiValueEncoded::new();
        // if let Some(additional_stake) = config.opt_additional_stake_required {
        //     for stake in additional_stake {
        //         additional_stake_as_tuple.push(MultiValue2::from((stake.token_id, stake.amount)));
        //     }
        // }

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainConfigContractProxy)
            .init(config, OWNER_ADDRESS)
            .code(CHAIN_CONFIG_CODE_PATH)
            .new_address(CHAIN_CONFIG_ADDRESS)
            .run();

        self
    }

    pub fn assert_expected_error_message(
        &mut self,
        response: Result<(), TxResponseStatus>,
        expected_error_message: Option<&str>,
    ) {
        match response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
            }
        }
    }

    // TODO: Add a better check balance for esdt function after check storage is fixed
    pub fn check_sc_esdt_balance<ContractObj>(
        &mut self,
        tokens: Vec<MultiValue3<TestTokenIdentifier, u64, u64>>,
        contract_address: ManagedAddress<StaticApi>,
        contract: fn() -> ContractObj,
    ) where
        ContractObj: ContractBase<Api = DebugApi> + 'static,
    {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(contract_address)
            .whitebox(contract, |sc: ContractObj| {
                for token in tokens {
                    let (token_id, nonce, amount) = token.into_tuple();
                    let balance = sc
                        .blockchain()
                        .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), nonce);
                    assert_eq!(balance, BigUint::from(amount));
                }
            });
    }

    pub fn check_deposited_tokens_amount(&mut self, tokens: Vec<(TestTokenIdentifier, u64)>) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                for token in tokens {
                    let (token_id, amount) = token;
                    assert!(sc.deposited_tokens_amount(&token_id.into()).get() == amount);
                }
            });
    }

    pub fn check_multiversx_to_sovereign_token_id_mapper_is_empty(&mut self, token_name: &str) {
        self.world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                assert!(sc
                    .multiversx_to_sovereign_token_id_mapper(
                        &TestTokenIdentifier::new(token_name).into()
                    )
                    .is_empty());
            })
    }

    pub fn check_operation_hash_status_is_empty(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(sc
                    .operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .is_empty());
            },
        )
    }

    pub fn check_operation_hash_status(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
        status: OperationHashStatus,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(
                    sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                        .get()
                        == status
                );
            },
        )
    }

    pub fn assert_expected_log(&mut self, logs: Vec<Log>, expected_log: &str) {
        let expected_bytes = ManagedBuffer::<StaticApi>::from(expected_log).to_vec();

        let found_log = logs
            .iter()
            .find(|log| log.topics.iter().any(|topic| *topic == expected_bytes));

        assert!(found_log.is_some(), "Expected log not found");
    }
}
