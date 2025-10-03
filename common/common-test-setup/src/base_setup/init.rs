use crate::constants::*;
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{Address, BigUint, ManagedBuffer, MxscPath, TestTokenIdentifier, Vec},
    ScenarioWorld,
};

pub struct BaseSetup {
    pub world: ScenarioWorld,
    operation_nonce: u64,
}

pub struct AccountSetup<'a> {
    pub address: Address,
    pub code_path: Option<MxscPath<'a>>,
    pub esdt_balances: Option<Vec<(TestTokenIdentifier<'a>, u64, BigUint<StaticApi>)>>,
    pub egld_balance: Option<BigUint<StaticApi>>,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FEE_MARKET_CODE_PATH, mvx_fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);
    blockchain.register_contract(CHAIN_FACTORY_CODE_PATH, chain_factory::ContractBuilder);
    blockchain.register_contract(SOVEREIGN_FORGE_CODE_PATH, sovereign_forge::ContractBuilder);
    blockchain.register_contract(MVX_ESDT_SAFE_CODE_PATH, mvx_esdt_safe::ContractBuilder);
    blockchain.register_contract(SOV_FEE_MARKET_CODE_PATH, sov_fee_market::ContractBuilder);

    blockchain
}

impl BaseSetup {
    pub fn new(account_setups: Vec<AccountSetup>) -> Self {
        let mut world = world();

        for acc in account_setups {
            let mut acc_builder = match acc.code_path {
                Some(code_path) => world.account(acc.address.clone()).code(code_path).nonce(1),
                None => world.account(acc.address.clone()).nonce(1),
            };

            if let Some(esdt_balances) = &acc.esdt_balances {
                for (token_id, nonce, amount) in esdt_balances {
                    acc_builder = if *nonce != 0 {
                        acc_builder.esdt_nft_balance(
                            *token_id,
                            *nonce,
                            amount.clone(),
                            ManagedBuffer::new(),
                        )
                    } else {
                        acc_builder.esdt_balance(*token_id, amount.clone())
                    };
                }
            }

            if let Some(balance) = &acc.egld_balance {
                acc_builder.balance(balance.clone());
            }
        }

        Self {
            world,
            operation_nonce: 0,
        }
    }

    pub fn next_operation_nonce(&mut self) -> u64 {
        let current = self.operation_nonce;
        self.operation_nonce = self
            .operation_nonce
            .checked_add(1)
            .expect("operation nonce overflow");
        current
    }
}
