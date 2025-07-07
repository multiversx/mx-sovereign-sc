#![allow(non_snake_case)]
use common_interactor::common_sovereign_interactor::{
    EsdtSafeType, IssueTokenStruct, MintTokenStruct, TemplateAddresses,
};
use common_interactor::interactor_state::{AddressInfo, State};
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::{
    DEPLOY_COST, INTERACTOR_WORKING_DIR, NUMBER_OF_SHARDS, ONE_THOUSAND_TOKENS,
    PREFERRED_CHAIN_IDS, SOVEREIGN_FORGE_CODE_PATH,
};
use multiversx_sc_snippets::imports::*;
use proxies::sovereign_forge_proxy::SovereignForgeProxy;
use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;
use structs::forge::ScArray;

pub struct SovereignForgeInteract {
    pub interactor: Interactor,
    pub bridge_owner_shard_0: Address,
    pub bridge_owner_shard_1: Address,
    pub bridge_owner_shard_2: Address,
    pub sovereign_owner_shard_0: Address,
    pub sovereign_owner_shard_1: Address,
    pub sovereign_owner_shard_2: Address,
    pub bridge_service: Address,
    pub user_address: Address,
    pub second_user_address: Address,
    pub state: State,
}
impl CommonInteractorTrait for SovereignForgeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn sovereign_owner(&self) -> &Address {
        &self.sovereign_owner_shard_0
    }

    fn bridge_service(&self) -> &Address {
        &self.bridge_service
    }

    fn user_address(&self) -> &Address {
        &self.user_address
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }
}
impl SovereignForgeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config).await;
        interactor.initialize_tokens_in_wallets().await;
        interactor
    }

    async fn initialize_interactor(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        let current_working_dir = INTERACTOR_WORKING_DIR;
        interactor.set_current_dir_from_workspace(current_working_dir);

        let bridge_owner_shard_0 = interactor.register_wallet(test_wallets::bob()).await;
        let bridge_owner_shard_1 = interactor.register_wallet(test_wallets::alice()).await;
        let bridge_owner_shard_2 = interactor.register_wallet(test_wallets::carol()).await;
        let sovereign_owner_shard_0 = interactor.register_wallet(test_wallets::mike()).await;
        let sovereign_owner_shard_1 = interactor.register_wallet(test_wallets::frank()).await;
        let sovereign_owner_shard_2 = interactor.register_wallet(test_wallets::heidi()).await;
        let bridge_service = interactor.register_wallet(test_wallets::dan()).await; //shard 1
        let user_address = interactor.register_wallet(test_wallets::eve()).await; //shard 1
        let second_user_address = interactor.register_wallet(test_wallets::mallory()).await; //shard 1

        interactor.generate_blocks_until_epoch(1).await.unwrap();

        SovereignForgeInteract {
            interactor,
            bridge_owner_shard_0,
            bridge_owner_shard_1,
            bridge_owner_shard_2,
            sovereign_owner_shard_0,
            sovereign_owner_shard_1,
            sovereign_owner_shard_2,
            bridge_service,
            user_address,
            second_user_address,
            state: State::default(),
        }
    }

    async fn initialize_tokens_in_wallets(&mut self) {
        let first_token_struct = IssueTokenStruct {
            token_display_name: "MVX".to_string(),
            token_ticker: "MVX".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
        };
        let first_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let first_token = self
            .issue_and_mint_token(first_token_struct, first_token_mint)
            .await;
        self.state.set_first_token(first_token);

        let second_token_struct = IssueTokenStruct {
            token_display_name: "MVX2".to_string(),
            token_ticker: "MVX2".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
        };
        let second_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let second_token = self
            .issue_and_mint_token(second_token_struct, second_token_mint)
            .await;
        self.state.set_second_token(second_token);

        let fee_token_struct = IssueTokenStruct {
            token_display_name: "FEE".to_string(),
            token_ticker: "FEE".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
        };
        let fee_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let fee_token = self
            .issue_and_mint_token(fee_token_struct, fee_token_mint)
            .await;
        self.state.set_fee_token(fee_token);

        let nft_token_struct = IssueTokenStruct {
            token_display_name: "NFT".to_string(),
            token_ticker: "NFT".to_string(),
            token_type: EsdtTokenType::NonFungible,
            num_decimals: 0,
        };
        let nft_token_mint = MintTokenStruct {
            name: Some("NFT".to_string()),
            amount: BigUint::from(1u64),
            attributes: None,
        };
        let nft_token = self
            .issue_and_mint_token(nft_token_struct, nft_token_mint)
            .await;

        self.state.set_nft_token_id(nft_token);

        let sft_token_struct = IssueTokenStruct {
            token_display_name: "SFT".to_string(),
            token_ticker: "SFT".to_string(),
            token_type: EsdtTokenType::SemiFungible,
            num_decimals: 0,
        };
        let sft_token_mint = MintTokenStruct {
            name: Some("SFT".to_string()),
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let sft_token = self
            .issue_and_mint_token(sft_token_struct, sft_token_mint)
            .await;

        self.state.set_sft_token_id(sft_token);

        let dyn_token_struct = IssueTokenStruct {
            token_display_name: "DYN".to_string(),
            token_ticker: "DYN".to_string(),
            token_type: EsdtTokenType::DynamicNFT,
            num_decimals: 10,
        };
        let dyn_token_mint = MintTokenStruct {
            name: Some("DYN".to_string()),
            amount: BigUint::from(1u64),
            attributes: None,
        };

        let dyn_token = self
            .issue_and_mint_token(dyn_token_struct, dyn_token_mint)
            .await;

        self.state.set_dynamic_nft_token_id(dyn_token);

        let meta_esdt_token_struct = IssueTokenStruct {
            token_display_name: "META".to_string(),
            token_ticker: "META".to_string(),
            token_type: EsdtTokenType::Meta,
            num_decimals: 0,
        };
        let meta_esdt_token_mint = MintTokenStruct {
            name: Some("META".to_string()),
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };

        let meta_esdt_token = self
            .issue_and_mint_token(meta_esdt_token_struct, meta_esdt_token_mint)
            .await;

        self.state.set_meta_esdt_token_id(meta_esdt_token);
    }

    pub async fn deploy_and_complete_setup_phase(
        &mut self,
        deploy_cost: BigUint<StaticApi>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let initial_caller = self.bridge_owner_shard_0.clone();

        let sovereign_forge_address = self
            .deploy_sovereign_forge(initial_caller.clone(), &BigUint::from(DEPLOY_COST))
            .await;

        for shard_id in 0..NUMBER_OF_SHARDS {
            let caller = self.get_sovereign_owner_for_shard(shard_id);
            let template_contracts = self
                .deploy_template_contracts(caller.clone(), EsdtSafeType::MvxEsdtSafe)
                .await;

            let (
                chain_config_address,
                mvx_esdt_safe_address,
                fee_market_address,
                header_verifier_address,
            ) = match template_contracts.as_slice() {
                [a, b, c, d] => (a.clone(), b.clone(), c.clone(), d.clone()),
                _ => panic!(
                    "Expected 4 deployed contract addresses, got {}",
                    template_contracts.len()
                ),
            };

            self.finish_init_setup_phase_for_one_shard(
                shard_id,
                initial_caller.clone(),
                sovereign_forge_address.clone(),
                TemplateAddresses {
                    chain_config_address: chain_config_address.clone(),
                    header_verifier_address: header_verifier_address.clone(),
                    esdt_safe_address: mvx_esdt_safe_address.clone(),
                    fee_market_address: fee_market_address.clone(),
                },
            )
            .await;
            println!("Finished setup phase for shard {shard_id}");
        }

        for shard in 0..NUMBER_OF_SHARDS {
            self.deploy_on_one_shard(
                shard,
                deploy_cost.clone(),
                optional_esdt_safe_config.clone(),
                optional_sov_config.clone(),
                fee.clone(),
            )
            .await;
        }
    }

    pub async fn finish_init_setup_phase_for_one_shard(
        &mut self,
        shard_id: u32,
        initial_caller: Address,
        sovereign_forge_address: Address,
        template_addresses: TemplateAddresses,
    ) {
        let caller = self.get_bridge_owner_for_shard(shard_id);
        let preferred_chain_id = PREFERRED_CHAIN_IDS[shard_id as usize].to_string();

        self.deploy_chain_factory(
            caller.clone(),
            preferred_chain_id.clone(),
            sovereign_forge_address.clone(),
            template_addresses.clone(),
        )
        .await;
        self.register_chain_factory(initial_caller.clone(), shard_id, preferred_chain_id.clone())
            .await;

        self.deploy_token_handler(caller.clone(), preferred_chain_id.clone())
            .await;
        self.register_token_handler(initial_caller.clone(), shard_id, preferred_chain_id)
            .await;
    }

    pub async fn deploy_on_one_shard(
        &mut self,
        shard: u32,
        deploy_cost: BigUint<StaticApi>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let caller = self.get_sovereign_owner_for_shard(shard);
        let preferred_chain_id = PREFERRED_CHAIN_IDS[shard as usize].to_string();
        self.deploy_phase_one(
            caller.clone(),
            deploy_cost.clone(),
            Some(preferred_chain_id.clone().into()),
            optional_sov_config.clone(),
        )
        .await;
        self.deploy_phase_two(caller.clone(), optional_esdt_safe_config.clone())
            .await;
        self.deploy_phase_three(caller.clone(), fee.clone()).await;
        self.deploy_phase_four(caller.clone()).await;

        self.complete_setup_phase(caller.clone()).await;
        self.check_setup_phase_status(&preferred_chain_id, true)
            .await;

        self.update_smart_contracts_addresses_in_state(preferred_chain_id.clone())
            .await;

        self.deploy_testing_sc(caller.clone(), preferred_chain_id)
            .await;
    }

    pub async fn upgrade(&mut self, caller: Address) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_sovereign_forge_sc_address())
            .from(caller)
            .gas(50_000_000u64)
            .typed(SovereignForgeProxy)
            .upgrade()
            .code(SOVEREIGN_FORGE_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_token_handler(
        &mut self,
        caller: Address,
        shard_id: u32,
        chain_id: String,
    ) {
        let sovereign_forge_address = self.state.current_sovereign_forge_sc_address();
        let token_handler_address = self.state.get_token_handler_address(chain_id);
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_token_handler(shard_id, token_handler_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_chain_factory(
        &mut self,
        caller: Address,
        shard_id: u32,
        chain_id: String,
    ) {
        let sovereign_forge_address = self.state.current_sovereign_forge_sc_address();
        let chain_factory_address = self.state.get_chain_factory_sc_address(chain_id);

        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, chain_factory_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn update_smart_contracts_addresses_in_state(&mut self, chain_id: String) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .sovereign_deployed_contracts(chain_id.clone())
            .returns(ReturnsResult)
            .run()
            .await;

        for contract in result_value {
            let address = Bech32Address::from(contract.address.to_address());
            match contract.id {
                ScArray::ChainConfig => {
                    self.state.set_chain_config_sc_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                ScArray::ESDTSafe => {
                    self.state.set_mvx_esdt_safe_contract_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                ScArray::FeeMarket => {
                    self.state.set_fee_market_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                ScArray::HeaderVerifier => {
                    self.state.set_header_verifier_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                _ => {}
            }
        }
    }

    pub async fn check_setup_phase_status(&mut self, chain_id: &str, expected_value: bool) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .sovereign_setup_phase(chain_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        assert_eq!(
            result_value, expected_value,
            "Expected setup phase status to be {expected_value}, but got {result_value}"
        );
    }

    fn get_bridge_owner_for_shard(&self, shard_id: u32) -> Address {
        match shard_id {
            0 => self.bridge_owner_shard_0.clone(),
            1 => self.bridge_owner_shard_1.clone(),
            2 => self.bridge_owner_shard_2.clone(),
            _ => panic!("Invalid shard ID: {shard_id}"),
        }
    }

    fn get_sovereign_owner_for_shard(&self, shard_id: u32) -> Address {
        match shard_id {
            0 => self.sovereign_owner_shard_0.clone(),
            1 => self.sovereign_owner_shard_1.clone(),
            2 => self.sovereign_owner_shard_2.clone(),
            _ => panic!("Invalid shard ID: {shard_id}"),
        }
    }
}
