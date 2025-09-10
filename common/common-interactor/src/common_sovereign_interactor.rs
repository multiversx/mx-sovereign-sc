#![allow(async_fn_in_trait)]
use crate::{
    interactor_helpers::InteractorHelpers,
    interactor_state::{AddressInfo, EsdtTokenInfo},
    interactor_structs::{ActionConfig, IssueTokenStruct, MintTokenStruct, TemplateAddresses},
};
use common_test_setup::{
    base_setup::helpers::BLSKey,
    constants::{
        CHAIN_CONFIG_CODE_PATH, CHAIN_FACTORY_CODE_PATH, CHAIN_ID, DEPLOY_COST,
        FEE_MARKET_CODE_PATH, HEADER_VERIFIER_CODE_PATH, ISSUE_COST, MVX_ESDT_SAFE_CODE_PATH,
        NATIVE_TOKEN_NAME, NATIVE_TOKEN_TICKER, NUMBER_OF_SHARDS, SHARD_0,
        SOVEREIGN_FORGE_CODE_PATH, SOVEREIGN_TOKEN_PREFIX, TESTING_SC_CODE_PATH,
    },
};
use multiversx_sc::{
    codec::num_bigint,
    imports::{ESDTSystemSCProxy, OptionalValue, UserBuiltinProxy},
    types::{
        Address, BigUint, CodeMetadata, ESDTSystemSCAddress, EgldOrEsdtTokenIdentifier,
        EsdtTokenType, ManagedBuffer, ManagedVec, MultiEgldOrEsdtPayment, MultiValueEncoded,
        ReturnsNewAddress, ReturnsResult, ReturnsResultUnmanaged, TokenIdentifier,
    },
};
use multiversx_sc_snippets::{
    imports::{
        Bech32Address, ReturnsGasUsed, ReturnsHandledOrError, ReturnsLogs,
        ReturnsNewTokenIdentifier, StaticApi, Wallet,
    },
    multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256,
    test_wallets, InteractorRunAsync,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    chain_factory_proxy::ChainFactoryContractProxy,
    fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::{HeaderverifierProxy, OperationHashStatus},
    mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    sovereign_forge_proxy::SovereignForgeProxy,
    testing_sc_proxy::TestingScProxy,
};
use structs::{
    aliases::{OptionalValueTransferDataTuple, PaymentsVec},
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::{ContractInfo, NativeToken, ScArray},
    generate_hash::GenerateHash,
    operation::Operation,
    EsdtInfo, RegisterTokenOperation,
};

fn metadata() -> CodeMetadata {
    CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE
}

pub trait CommonInteractorTrait: InteractorHelpers {
    async fn register_wallets(&mut self) {
        let wallet_path = "wallets/wallet_shard_0.pem".to_string();
        let wallet = Wallet::from_pem_file(&wallet_path)
            .unwrap_or_else(|_| panic!("Failed to load wallet for shard 0"));

        self.interactor().register_wallet(test_wallets::bob()).await;
        self.interactor().register_wallet(test_wallets::dan()).await;
        self.interactor()
            .register_wallet(test_wallets::heidi())
            .await;

        self.interactor()
            .register_wallet(test_wallets::mike())
            .await;
        self.interactor().register_wallet(test_wallets::eve()).await;
        self.interactor()
            .register_wallet(test_wallets::judy())
            .await;

        self.interactor().register_wallet(wallet).await;
        self.interactor()
            .register_wallet(test_wallets::frank())
            .await;
        self.interactor()
            .register_wallet(test_wallets::carol())
            .await;

        self.interactor().generate_blocks(2u64).await.unwrap();
    }

    async fn issue_and_mint_token(
        &mut self,
        issue: IssueTokenStruct,
        mint: MintTokenStruct,
    ) -> EsdtTokenInfo {
        let user_address = self.user_address().clone();
        let interactor = self.interactor();

        let token_id = interactor
            .tx()
            .from(user_address)
            .to(ESDTSystemSCAddress)
            .gas(100_000_000u64)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(
                ISSUE_COST.into(),
                issue.token_display_name,
                issue.token_ticker,
                issue.token_type,
                issue.num_decimals,
            )
            .returns(ReturnsNewTokenIdentifier)
            .run()
            .await;

        let nonce = self
            .mint_tokens(token_id.clone(), issue.token_type, mint.clone())
            .await;

        let decimals = self.get_token_decimals(issue.token_type);

        EsdtTokenInfo {
            token_id: EgldOrEsdtTokenIdentifier::from(token_id.as_bytes()),
            nonce,
            token_type: issue.token_type,
            decimals,
            amount: mint.amount,
        }
    }

    async fn mint_tokens(
        &mut self,
        token_id: String,
        token_type: EsdtTokenType,
        mint: MintTokenStruct,
    ) -> u64 {
        let user_address = self.user_address().clone();
        let interactor = self.interactor();
        let mint_base_tx = interactor
            .tx()
            .from(user_address.clone())
            .to(user_address)
            .gas(100_000_000u64)
            .typed(UserBuiltinProxy);

        match token_type {
            EsdtTokenType::Fungible => {
                mint_base_tx
                    .esdt_local_mint(TokenIdentifier::from(token_id.as_bytes()), 0, mint.amount)
                    .returns(ReturnsResultUnmanaged)
                    .run()
                    .await;
                0u64
            }
            EsdtTokenType::NonFungibleV2
            | EsdtTokenType::SemiFungible
            | EsdtTokenType::DynamicNFT
            | EsdtTokenType::DynamicMeta
            | EsdtTokenType::DynamicSFT
            | EsdtTokenType::MetaFungible => {
                mint_base_tx
                    .esdt_nft_create(
                        TokenIdentifier::from(token_id.as_bytes()),
                        mint.amount,
                        mint.name.unwrap_or_default(),
                        BigUint::zero(),
                        ManagedBuffer::new(),
                        &mint.attributes.unwrap_or_default(),
                        &ManagedVec::new(),
                    )
                    .returns(ReturnsResult)
                    .run()
                    .await
            }
            _ => {
                panic!("Unsupported token type: {:?}", token_type);
            }
        }
    }

    async fn create_token_with_config(
        &mut self,
        token_type: EsdtTokenType,
        ticker: &str,
        amount: BigUint<StaticApi>,
        decimals: usize,
    ) -> EsdtTokenInfo {
        let token_struct = IssueTokenStruct {
            token_display_name: ticker.to_string(),
            token_ticker: ticker.to_string(),
            token_type,
            num_decimals: decimals,
        };

        let mint_struct = MintTokenStruct {
            name: if matches!(token_type, EsdtTokenType::Fungible) {
                None
            } else {
                Some(ticker.to_string())
            },
            amount,
            attributes: None,
        };

        self.issue_and_mint_token(token_struct, mint_struct).await
    }

    async fn deploy_sovereign_forge(
        &mut self,
        caller: Address,
        deploy_cost: OptionalValue<&BigUint<StaticApi>>,
    ) -> Address {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(70_000_000u64)
            .typed(SovereignForgeProxy)
            .init(deploy_cost)
            .code(SOVEREIGN_FORGE_CODE_PATH)
            .code_metadata(metadata())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.common_state()
            .set_sovereign_forge_sc_address(new_address_bech32.clone());

        new_address
    }

    async fn deploy_chain_factory(
        &mut self,
        caller: Address,
        sovereign_forge_address: Address,
        template_addresses: TemplateAddresses,
    ) {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(50_000_000u64)
            .typed(ChainFactoryContractProxy)
            .init(
                sovereign_forge_address,
                template_addresses.chain_config_address,
                template_addresses.header_verifier_address,
                template_addresses.esdt_safe_address,
                template_addresses.fee_market_address,
            )
            .code(CHAIN_FACTORY_CODE_PATH)
            .code_metadata(metadata())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.common_state()
            .set_chain_factory_sc_address(new_address_bech32);
    }

    async fn deploy_chain_config(
        &mut self,
        caller: Address,
        chain_id: String,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
    ) {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(50_000_000u64)
            .typed(ChainConfigContractProxy)
            .init(opt_config)
            .returns(ReturnsNewAddress)
            .code(CHAIN_CONFIG_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.common_state()
            .set_chain_config_sc_address(AddressInfo {
                address: new_address_bech32,
                chain_id,
            });
    }

    async fn deploy_template_contracts(
        &mut self,
        caller: Address,
        chain_id: &str,
    ) -> Vec<Bech32Address> {
        let mut template_contracts = vec![];

        let chain_config_template = self
            .interactor()
            .tx()
            .from(caller.clone())
            .gas(50_000_000u64)
            .typed(ChainConfigContractProxy)
            .init(OptionalValue::<SovereignConfig<StaticApi>>::None)
            .returns(ReturnsNewAddress)
            .code(CHAIN_CONFIG_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;
        template_contracts.push(Bech32Address::from(chain_config_template));

        let esdt_safe_template = self
            .interactor()
            .tx()
            .from(caller.clone())
            .gas(100_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(chain_id, OptionalValue::<EsdtSafeConfig<StaticApi>>::None)
            .returns(ReturnsNewAddress)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;
        template_contracts.push(Bech32Address::from(esdt_safe_template.clone()));

        let fee_market_address = self
            .interactor()
            .tx()
            .from(caller.clone())
            .gas(80_000_000u64)
            .typed(FeeMarketProxy)
            .init(
                Bech32Address::from(esdt_safe_template),
                None::<FeeStruct<StaticApi>>,
            )
            .returns(ReturnsNewAddress)
            .code(FEE_MARKET_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;
        template_contracts.push(Bech32Address::from(fee_market_address));

        let header_verifier_address = self
            .interactor()
            .tx()
            .from(caller.clone())
            .gas(50_000_000u64)
            .typed(HeaderverifierProxy)
            .init(MultiValueEncoded::new())
            .returns(ReturnsNewAddress)
            .code(HEADER_VERIFIER_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;
        template_contracts.push(Bech32Address::from(header_verifier_address));

        template_contracts
    }

    async fn deploy_header_verifier(
        &mut self,
        caller: Address,
        chain_id: String,
        contracts_array: Vec<ContractInfo<StaticApi>>,
    ) {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(50_000_000u64)
            .typed(HeaderverifierProxy)
            .init(MultiValueEncoded::from_iter(contracts_array))
            .returns(ReturnsNewAddress)
            .code(HEADER_VERIFIER_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.common_state()
            .set_header_verifier_address(AddressInfo {
                address: new_address_bech32,
                chain_id,
            });
    }

    async fn deploy_mvx_esdt_safe(
        &mut self,
        caller: Address,
        chain_id: String,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(100_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(SOVEREIGN_TOKEN_PREFIX, opt_config)
            .returns(ReturnsNewAddress)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.common_state()
            .set_mvx_esdt_safe_contract_address(AddressInfo {
                address: new_address_bech32.clone(),
                chain_id,
            });
    }

    async fn register_as_validator(
        &mut self,
        shard: u32,
        bls_key: ManagedBuffer<StaticApi>,
        payment: MultiEgldOrEsdtPayment<StaticApi>,
        chain_config_address: Bech32Address,
    ) {
        let bridge_owner = self.get_bridge_owner_for_shard(shard).clone();

        self.interactor()
            .tx()
            .from(bridge_owner)
            .to(chain_config_address)
            .gas(90_000_000u64)
            .typed(ChainConfigContractProxy)
            .register(bls_key)
            .payment(payment)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn deploy_fee_market(
        &mut self,
        caller: Address,
        chain_id: String,
        esdt_safe_address: Bech32Address,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(80_000_000u64)
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .returns(ReturnsNewAddress)
            .code(FEE_MARKET_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.common_state().set_fee_market_address(AddressInfo {
            address: new_address_bech32.clone(),
            chain_id,
        });
    }

    async fn deploy_testing_sc(&mut self) {
        let bridge_owner = self.get_bridge_owner_for_shard(SHARD_0).clone();
        let new_address = self
            .interactor()
            .tx()
            .from(bridge_owner)
            .gas(120_000_000u64)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .code_metadata(metadata())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);

        self.common_state()
            .set_testing_sc_address(new_address_bech32.clone());

        println!("new testing sc address: {new_address_bech32}");
    }

    async fn deploy_and_complete_setup_phase(
        &mut self,
        deploy_cost: OptionalValue<BigUint<StaticApi>>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) {
        let fee_struct = self.create_standard_fee();
        self.deploy_and_setup_common(
            deploy_cost.clone(),
            optional_sov_config,
            optional_esdt_safe_config,
            Some(fee_struct),
        )
        .await;
        let fee_token_id = self.state().get_fee_token_id();
        let fee_token_fee_market = self.create_fee_market_token_state(fee_token_id, 0u64).await;
        self.common_state()
            .set_fee_market_token_for_all_shards(fee_token_fee_market);
        self.common_state().set_fee_status_for_all_shards(true);
        self.common_state().fee_op_nonce = 1u64;
    }

    async fn deploy_and_setup_common(
        &mut self,
        deploy_cost: OptionalValue<BigUint<StaticApi>>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let initial_caller = self.get_bridge_owner_for_shard(SHARD_0).clone();

        let sovereign_forge_address = self
            .deploy_sovereign_forge(
                initial_caller.clone(),
                OptionalValue::Some(&BigUint::from(DEPLOY_COST)),
            )
            .await;

        for shard_id in 0..NUMBER_OF_SHARDS {
            let caller = self.get_bridge_owner_for_shard(shard_id);
            let template_contracts = self
                .deploy_template_contracts(caller.clone(), CHAIN_ID)
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

        self.deploy_testing_sc().await;
    }

    async fn finish_init_setup_phase_for_one_shard(
        &mut self,
        shard_id: u32,
        initial_caller: Address,
        sovereign_forge_address: Address,
        template_addresses: TemplateAddresses,
    ) {
        let caller = self.get_bridge_owner_for_shard(shard_id);

        self.deploy_chain_factory(
            caller.clone(),
            sovereign_forge_address.clone(),
            template_addresses.clone(),
        )
        .await;
        self.register_chain_factory(initial_caller.clone(), shard_id)
            .await;
    }

    async fn deploy_on_one_shard(
        &mut self,
        shard: u32,
        deploy_cost: OptionalValue<BigUint<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let caller = self.get_sovereign_owner_for_shard(shard);
        let preferred_chain_id = Self::generate_random_chain_id();
        self.common_state().add_chain_id(preferred_chain_id.clone());
        self.deploy_phase_one(
            caller.clone(),
            deploy_cost.clone(),
            Some(preferred_chain_id.clone().into()),
            optional_sov_config.clone(),
        )
        .await;
        let chain_config_address = self.get_chain_config_address(&preferred_chain_id).await;
        self.register_as_validator(
            shard,
            BLSKey::random(),
            MultiEgldOrEsdtPayment::new(),
            chain_config_address,
        )
        .await;
        self.deploy_phase_two(optional_esdt_safe_config.clone(), caller.clone())
            .await;
        // self.register_native_token(caller.clone()).await;
        self.deploy_phase_three(caller.clone(), fee.clone()).await;
        self.deploy_phase_four(caller.clone()).await;

        self.complete_setup_phase(caller.clone()).await;
        self.check_setup_phase_status(&preferred_chain_id, true)
            .await;

        self.update_smart_contracts_addresses_in_state(preferred_chain_id.clone())
            .await;
        println!("Finished deployment for shard {shard}");
    }

    async fn register_chain_factory(&mut self, caller: Address, shard_id: u32) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();
        let chain_factory_address = self
            .common_state()
            .get_chain_factory_sc_address(shard_id)
            .clone();

        let response = self
            .interactor()
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

    async fn register_native_token(&mut self, caller: Address) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();
        let native_token = NativeToken {
            name: ManagedBuffer::from(NATIVE_TOKEN_NAME),
            ticker: ManagedBuffer::from(NATIVE_TOKEN_TICKER),
        };
        self.interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(80_000_000u64)
            .typed(SovereignForgeProxy)
            .register_native_token(native_token)
            .egld(ISSUE_COST)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn update_smart_contracts_addresses_in_state(&mut self, chain_id: String) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        let result_value = self
            .interactor()
            .query()
            .to(sovereign_forge_address)
            .typed(SovereignForgeProxy)
            .sovereign_deployed_contracts(chain_id.clone())
            .returns(ReturnsResult)
            .run()
            .await;

        for contract in result_value {
            let address = Bech32Address::from(contract.address.to_address());
            match contract.id {
                ScArray::ChainConfig => {
                    self.common_state()
                        .set_chain_config_sc_address(AddressInfo {
                            address,
                            chain_id: chain_id.clone(),
                        });
                }
                ScArray::ESDTSafe => {
                    self.common_state()
                        .set_mvx_esdt_safe_contract_address(AddressInfo {
                            address,
                            chain_id: chain_id.clone(),
                        });
                }
                ScArray::FeeMarket => {
                    self.common_state().set_fee_market_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                ScArray::HeaderVerifier => {
                    self.common_state()
                        .set_header_verifier_address(AddressInfo {
                            address,
                            chain_id: chain_id.clone(),
                        });
                }
                _ => {}
            }
        }
    }

    async fn get_chain_config_address(&mut self, chain_id: &str) -> Bech32Address {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        let result_value = self
            .interactor()
            .query()
            .to(sovereign_forge_address)
            .typed(SovereignForgeProxy)
            .sovereign_deployed_contracts(chain_id)
            .returns(ReturnsResult)
            .run()
            .await;

        for contract in result_value {
            if let ScArray::ChainConfig = contract.id {
                return Bech32Address::from(contract.address.to_address());
            }
        }

        panic!("Chain config address not found for chain_id: {}", chain_id);
    }

    async fn deploy_phase_one(
        &mut self,
        caller: Address,
        opt_egld_amount: OptionalValue<BigUint<StaticApi>>,
        opt_preferred_chain_id: Option<ManagedBuffer<StaticApi>>,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
    ) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        let mut egld_amount = BigUint::default();

        if opt_egld_amount.is_some() {
            egld_amount = opt_egld_amount.into_option().unwrap();
        }

        let response = self
            .interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_one(opt_preferred_chain_id, opt_config)
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deploy_phase_two(
        &mut self,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        caller: Address,
    ) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();
        let response = self
            .interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_two(opt_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deploy_phase_three(&mut self, caller: Address, fee: Option<FeeStruct<StaticApi>>) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        let response = self
            .interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_three(fee)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deploy_phase_four(&mut self, caller: Address) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        let response = self
            .interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_four()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn complete_setup_phase(&mut self, caller: Address) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        self.interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(90_000_000u64)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsGasUsed)
            .run()
            .await;
    }

    async fn update_esdt_safe_config(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        new_config: EsdtSafeConfig<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_mvx_esdt_safe_address =
            self.common_state().get_mvx_esdt_safe_address(shard).clone();

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(current_mvx_esdt_safe_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_esdt_safe_config(hash_of_hashes, new_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn set_fee_after_setup_phase(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        fee: FeeStruct<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_fee_market_address = self.common_state().get_fee_market_address(shard).clone();

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(current_fee_market_address)
            .gas(50_000_000u64)
            .typed(FeeMarketProxy)
            .set_fee(hash_of_hashes, fee)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn remove_fee_after_setup_phase(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        base_token: EgldOrEsdtTokenIdentifier<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_fee_market_address = self.common_state().get_fee_market_address(shard).clone();

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(current_fee_market_address)
            .gas(50_000_000u64)
            .typed(FeeMarketProxy)
            .remove_fee(hash_of_hashes, base_token)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn set_token_burn_mechanism(&mut self, token_id: TokenIdentifier<StaticApi>) {
        let current_mvx_esdt_safe_address = self
            .common_state()
            .current_mvx_esdt_safe_contract_address()
            .clone();
        let sovereign_owner = self.get_sovereign_owner_for_shard(SHARD_0).clone();

        self.interactor()
            .tx()
            .to(current_mvx_esdt_safe_address)
            .from(sovereign_owner)
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_token_burn_mechanism(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn register_operation(
        &mut self,
        shard: u32,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let header_verifier_address = self
            .common_state()
            .get_header_verifier_address(shard)
            .clone();

        let bitmap = ManagedBuffer::new_from_bytes(&[1]);
        let epoch = 0u32;

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(header_verifier_address)
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(signature, hash_of_hashes, bitmap, epoch, operations_hashes)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn complete_header_verifier_setup_phase(&mut self, caller: Address) {
        let header_verifier_address = self
            .common_state()
            .current_header_verifier_address()
            .clone();

        self.interactor()
            .tx()
            .from(caller)
            .to(header_verifier_address)
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn complete_chain_config_setup_phase(&mut self, shard: u32) {
        let bridge_owner = self.get_bridge_owner_for_shard(shard).clone();
        let chain_config_address = self
            .common_state()
            .current_chain_config_sc_address()
            .clone();

        self.interactor()
            .tx()
            .from(bridge_owner)
            .to(chain_config_address)
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn deposit_in_mvx_esdt_safe(
        &mut self,
        to: Address,
        shard: u32,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payments: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let user_address = self.user_address().clone();
        let current_mvx_esdt_safe_address =
            self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let (response, logs) = self
            .interactor()
            .tx()
            .from(user_address)
            .to(current_mvx_esdt_safe_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        self.assert_expected_log(logs, expected_log, None);
    }

    async fn withdraw_from_testing_sc(
        &mut self,
        expected_token: EsdtTokenInfo,
        nonce: u64,
        amount: BigUint<StaticApi>,
    ) {
        let user_address = self.user_address().clone();
        let testing_sc_address = self.common_state().current_testing_sc_address().clone();
        self.interactor()
            .tx()
            .from(user_address)
            .to(testing_sc_address)
            .gas(90_000_000u64)
            .typed(TestingScProxy)
            .send_tokens(expected_token.token_id, nonce, amount.clone())
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_operations_in_mvx_esdt_safe(
        &mut self,
        caller: Address,
        shard: u32,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        let current_mvx_esdt_safe_address =
            self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let (response, logs) = self
            .interactor()
            .tx()
            .from(caller)
            .to(current_mvx_esdt_safe_address)
            .gas(120_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        self.assert_expected_log(logs, expected_log, expected_log_error);
    }

    async fn register_token(
        &mut self,
        shard: u32,
        token: RegisterTokenOperation<StaticApi>,
        expected_log_error: Option<&str>,
    ) -> Option<String> {
        let user_address = self.user_address().clone();
        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let token_hash = token.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&token_hash.to_vec()));

        let base_transaction = self
            .interactor()
            .tx()
            .from(user_address)
            .to(mvx_esdt_safe_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .register_token(hash_of_hashes, token)
            .returns(ReturnsLogs);

        let (response, token) = match expected_log_error {
            Some(_) => {
                let response = base_transaction.run().await;
                (response, None)
            }
            None => {
                let (response, token) = base_transaction
                    .returns(ReturnsNewTokenIdentifier)
                    .run()
                    .await;
                (response, Some(token))
            }
        };

        self.assert_expected_log(response, Some(""), expected_log_error);
        token
    }

    async fn get_sov_to_mvx_token_id(
        &mut self,
        shard: u32,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
    ) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let user_address = self.user_address().clone();
        self.interactor()
            .tx()
            .from(user_address)
            .to(mvx_esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .sovereign_to_multiversx_token_id_mapper(token_id)
            .returns(ReturnsResult)
            .run()
            .await
    }

    async fn get_sov_to_mvx_token_id_with_nonce(
        &mut self,
        shard: u32,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
        nonce: u64,
    ) -> EsdtInfo<StaticApi> {
        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let user_address = self.user_address().clone();
        self.interactor()
            .tx()
            .from(user_address)
            .to(mvx_esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .sovereign_to_multiversx_esdt_info_mapper(token_id, nonce)
            .returns(ReturnsResult)
            .run()
            .await
    }

    async fn check_setup_phase_status(&mut self, chain_id: &str, expected_value: bool) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();
        let result_value = self
            .interactor()
            .query()
            .to(sovereign_forge_address)
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

    async fn check_registered_operation_status(
        &mut self,
        shard_id: u32,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation_hash: ManagedBuffer<StaticApi>,
        expected_value: OperationHashStatus,
    ) {
        let header_verifier_address = self
            .common_state()
            .get_header_verifier_address(shard_id)
            .clone();
        let response = self
            .interactor()
            .query()
            .to(header_verifier_address)
            .typed(HeaderverifierProxy)
            .operation_hash_status(hash_of_hashes, operation_hash)
            .returns(ReturnsResult)
            .run()
            .await;

        assert_eq!(
            response, expected_value,
            "Expected operation hash status {:?} does not match with the actual value {:?}",
            expected_value, response
        );
    }

    async fn get_mapped_token(
        &mut self,
        config: ActionConfig,
        original_token: &EsdtTokenInfo,
        amount: &BigUint<StaticApi>,
    ) -> EsdtTokenInfo {
        let edge_case = original_token.token_type == EsdtTokenType::Fungible
            || (self.is_nft(original_token) && config.expected_error.is_some());

        let (mapped_token_id, mapped_nonce) = if edge_case {
            let token_id = self
                .get_sov_to_mvx_token_id(config.shard, original_token.clone().token_id)
                .await;
            (token_id, original_token.nonce)
        } else {
            let token_info = self
                .get_sov_to_mvx_token_id_with_nonce(
                    config.shard,
                    original_token.clone().token_id,
                    original_token.nonce,
                )
                .await;
            (token_info.token_identifier, token_info.token_nonce)
        };

        EsdtTokenInfo {
            token_id: mapped_token_id,
            nonce: mapped_nonce,
            token_type: original_token.token_type,
            decimals: original_token.decimals,
            amount: amount.clone(),
        }
    }

    async fn retrieve_current_fee_token_for_wallet(&mut self) -> EsdtTokenInfo {
        let fee_token_id = self
            .common_state()
            .fee_market_tokens
            .get("0")
            .map(|t| t.token_id.clone())
            .expect("Fee market token for shard 0 not found");

        let user_address = &self.user_address().clone();
        let balances = self.interactor().get_account_esdt(user_address).await;

        let amount = if let Some(esdt_balance) = balances.get(&fee_token_id) {
            BigUint::from(
                num_bigint::BigUint::parse_bytes(esdt_balance.balance.as_bytes(), 10)
                    .expect("Failed to parse fee token balance as number"),
            )
        } else {
            BigUint::zero()
        };

        EsdtTokenInfo {
            token_id: EgldOrEsdtTokenIdentifier::from(fee_token_id.as_str()),
            nonce: 0,
            token_type: EsdtTokenType::Fungible,
            amount,
            decimals: 18,
        }
    }

    async fn set_fee(&mut self, fee: FeeStruct<StaticApi>, shard: u32) {
        let fee_activated = self.common_state().get_fee_status_for_shard(shard);

        if fee_activated {
            return;
        }
        self.set_fee_common(fee, shard).await;
    }

    async fn remove_fee(&mut self, shard: u32) {
        let fee_activated = self.common_state().get_fee_status_for_shard(shard);

        if !fee_activated {
            return;
        }
        self.common_state().fee_op_nonce += 1;
        let nonce_str = self.common_state().fee_op_nonce.to_string();
        let nonce_buf = ManagedBuffer::<StaticApi>::from(&nonce_str);

        let fee_token = self.state().get_fee_token_identifier();
        let token_hash = fee_token.generate_hash();

        let mut bytes = Vec::with_capacity(token_hash.len() + nonce_buf.len());
        bytes.extend_from_slice(&token_hash.to_vec());
        bytes.extend_from_slice(&nonce_buf.to_vec());

        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&bytes));
        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![token_hash.clone(), nonce_buf]));

        self.register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

        self.remove_fee_after_setup_phase(hash_of_hashes, fee_token, shard)
            .await;
        self.common_state().set_fee_status_for_shard(shard, false);
    }

    async fn set_fee_common(&mut self, fee: FeeStruct<StaticApi>, shard: u32) {
        self.common_state().fee_op_nonce += 1;
        let nonce_str = self.common_state().fee_op_nonce.to_string();
        let nonce_buf = ManagedBuffer::<StaticApi>::from(&nonce_str);

        let fee_hash = fee.generate_hash();

        let mut bytes = Vec::with_capacity(fee_hash.len() + nonce_buf.len());
        bytes.extend_from_slice(&fee_hash.to_vec());
        bytes.extend_from_slice(&nonce_buf.to_vec());

        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&bytes));
        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![fee_hash.clone(), nonce_buf]));

        self.register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

        self.set_fee_after_setup_phase(hash_of_hashes, fee, shard)
            .await;
        self.common_state().set_fee_status_for_shard(shard, true);
    }
}
