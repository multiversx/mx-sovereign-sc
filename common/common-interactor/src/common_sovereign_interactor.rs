#![allow(async_fn_in_trait)]

use crate::interactor_state::{AddressInfo, EsdtTokenInfo, State};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use common_test_setup::constants::{
    CHAIN_CONFIG_CODE_PATH, CHAIN_FACTORY_CODE_PATH, DEPLOY_COST, ENSHRINE_ESDT_SAFE_CODE_PATH,
    FEE_MARKET_CODE_PATH, HEADER_VERIFIER_CODE_PATH, ISSUE_COST, MVX_ESDT_SAFE_CODE_PATH,
    NUMBER_OF_SHARDS, PER_GAS, PER_TRANSFER, PREFERRED_CHAIN_IDS, SHARD_0,
    SOVEREIGN_FORGE_CODE_PATH, SOVEREIGN_TOKEN_PREFIX, TESTING_SC_CODE_PATH,
    TOKEN_HANDLER_CODE_PATH, WEGLD_IDENTIFIER,
};
use error_messages::{FAILED_TO_LOAD_WALLET_SHARD_0, FAILED_TO_PARSE_AS_NUMBER};
use multiversx_sc::{
    codec::{num_bigint, TopEncode},
    imports::{ESDTSystemSCProxy, OptionalValue, UserBuiltinProxy},
    types::{
        Address, BigUint, CodeMetadata, ESDTSystemSCAddress, EsdtTokenType, ManagedAddress,
        ManagedBuffer, ManagedVec, MultiValueEncoded, ReturnsNewAddress, ReturnsResult,
        ReturnsResultUnmanaged, TestSCAddress, TokenIdentifier,
    },
};
use multiversx_sc_snippets::{
    hex,
    imports::{
        Bech32Address, ReturnsGasUsed, ReturnsHandledOrError, ReturnsLogs,
        ReturnsNewTokenIdentifier, StaticApi, Wallet,
    },
    multiversx_sc_scenario::{
        multiversx_chain_vm::crypto_functions::sha256,
        scenario_model::{Log, TxResponseStatus},
    },
    test_wallets, Interactor, InteractorRunAsync,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
    enshrine_esdt_safe_proxy, fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::HeaderverifierProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    sovereign_forge_proxy::SovereignForgeProxy, testing_sc_proxy::TestingScProxy,
    token_handler_proxy,
};
use structs::{
    aliases::{OptionalValueTransferDataTuple, PaymentsVec},
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::{FeeStruct, FeeType},
    forge::{ContractInfo, ScArray},
    operation::Operation,
    EsdtInfo,
};

use common_test_setup::base_setup::init::RegisterTokenArgs;

fn metadata() -> CodeMetadata {
    CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE
}

pub struct IssueTokenStruct {
    pub token_display_name: String,
    pub token_ticker: String,
    pub token_type: EsdtTokenType,
    pub num_decimals: usize,
}
#[derive(Clone)]
pub struct MintTokenStruct {
    pub name: Option<String>,
    pub amount: BigUint<StaticApi>,
    pub attributes: Option<Vec<u8>>,
}

pub enum EsdtSafeType {
    MvxEsdtSafe,
    EnshrineEsdtSafe,
}

#[derive(Clone)]
pub struct TemplateAddresses {
    pub chain_config_address: Bech32Address,
    pub header_verifier_address: Bech32Address,
    pub esdt_safe_address: Bech32Address,
    pub fee_market_address: Bech32Address,
}

pub trait CommonInteractorTrait {
    fn interactor(&mut self) -> &mut Interactor;
    fn state(&mut self) -> &mut State;
    fn user_address(&self) -> &Address;

    async fn register_wallets(&mut self) {
        let shard_0_wallet = Wallet::from_pem_file("wallets/shard-0-wallet.pem")
            .expect(FAILED_TO_LOAD_WALLET_SHARD_0);

        self.interactor().register_wallet(test_wallets::bob()).await; // bridge_owner_shard_0
        self.interactor()
            .register_wallet(test_wallets::alice())
            .await; // bridge_owner_shard_1
        self.interactor()
            .register_wallet(test_wallets::carol())
            .await; // bridge_owner_shard_2
        self.interactor()
            .register_wallet(test_wallets::mike())
            .await; // sovereign_owner_shard_0
        self.interactor()
            .register_wallet(test_wallets::frank())
            .await; // sovereign_owner_shard_1
        self.interactor()
            .register_wallet(test_wallets::heidi())
            .await; // sovereign_owner_shard_2
        self.interactor().register_wallet(shard_0_wallet).await; // bridge_service_shard_0
        self.interactor().register_wallet(test_wallets::dan()).await; // bridge_service_shard_1
        self.interactor()
            .register_wallet(test_wallets::judy())
            .await; // bridge_service_shard_2

        self.interactor().generate_blocks(1u64).await.unwrap();
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

        EsdtTokenInfo {
            token_id: token_id.clone(),
            nonce,
            token_type: issue.token_type,
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

    async fn deploy_sovereign_forge(
        &mut self,
        caller: Address,
        deploy_cost: &BigUint<StaticApi>,
    ) -> Address {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(50_000_000u64)
            .typed(SovereignForgeProxy)
            .init(deploy_cost)
            .code(SOVEREIGN_FORGE_CODE_PATH)
            .code_metadata(metadata())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.state()
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
        self.state()
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
        self.state().set_chain_config_sc_address(AddressInfo {
            address: new_address_bech32,
            chain_id,
        });
    }

    async fn deploy_template_contracts(
        &mut self,
        caller: Address,
        esdt_safe_type: EsdtSafeType,
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

        let esdt_safe_template = match esdt_safe_type {
            EsdtSafeType::MvxEsdtSafe => {
                let mvx_esdt_safe_template = self
                    .interactor()
                    .tx()
                    .from(caller.clone())
                    .gas(100_000_000u64)
                    .typed(MvxEsdtSafeProxy)
                    .init(OptionalValue::<EsdtSafeConfig<StaticApi>>::None)
                    .returns(ReturnsNewAddress)
                    .code(MVX_ESDT_SAFE_CODE_PATH)
                    .code_metadata(metadata())
                    .run()
                    .await;
                template_contracts.push(Bech32Address::from(mvx_esdt_safe_template.clone()));
                mvx_esdt_safe_template
            }
            EsdtSafeType::EnshrineEsdtSafe => {
                let enshrine_esdt_safe_template = self
                    .interactor()
                    .tx()
                    .from(caller.clone())
                    .gas(100_000_000u64)
                    .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                    .init(
                        false,
                        ESDTSystemSCAddress,
                        Some(TokenIdentifier::from(WEGLD_IDENTIFIER)),
                        Some(ManagedBuffer::from(SOVEREIGN_TOKEN_PREFIX)),
                        None::<EsdtSafeConfig<StaticApi>>,
                    )
                    .returns(ReturnsNewAddress)
                    .code(ENSHRINE_ESDT_SAFE_CODE_PATH)
                    .code_metadata(metadata())
                    .run()
                    .await;
                template_contracts.push(Bech32Address::from(enshrine_esdt_safe_template.clone()));
                enshrine_esdt_safe_template
            }
        };

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
        self.state().set_header_verifier_address(AddressInfo {
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
            .init(opt_config)
            .returns(ReturnsNewAddress)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(metadata())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.state()
            .set_mvx_esdt_safe_contract_address(AddressInfo {
                address: new_address_bech32.clone(),
                chain_id,
            });
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
        self.state().set_fee_market_address(AddressInfo {
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
        self.state().set_testing_sc_address(new_address_bech32);
    }

    async fn deploy_token_handler(&mut self, caller: Address, shard: u32) {
        let chain_factory_address = self.state().get_chain_factory_sc_address(shard).clone();

        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(100_000_000u64)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init(chain_factory_address)
            .code(TOKEN_HANDLER_CODE_PATH)
            .code_metadata(metadata())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.state().set_token_handler_address(new_address_bech32);
    }

    #[allow(clippy::too_many_arguments)]
    async fn deploy_enshrine_esdt(
        &mut self,
        caller: Address,
        shard: u32,
        chain_id: String,
        is_sovereign_chain: bool,
        opt_wegld_identifier: Option<TokenIdentifier<StaticApi>>,
        opt_sov_token_prefix: Option<ManagedBuffer<StaticApi>>,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
    ) {
        let token_handler_address = self.state().get_token_handler_address(shard).clone();
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(100_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                token_handler_address,
                opt_wegld_identifier,
                opt_sov_token_prefix,
                opt_config,
            )
            .code(ENSHRINE_ESDT_SAFE_CODE_PATH)
            .code_metadata(metadata())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from(&new_address);
        self.state().set_enshrine_esdt_safe_sc_address(AddressInfo {
            address: new_address_bech32,
            chain_id,
        });
    }

    async fn deploy_and_complete_setup_phase(
        &mut self,
        deploy_cost: BigUint<StaticApi>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        self.deploy_and_setup_common(
            deploy_cost.clone(),
            optional_sov_config,
            optional_esdt_safe_config,
            fee,
            None,
        )
        .await;
    }

    async fn deploy_and_complete_setup_phase_on_a_shard(
        &mut self,
        shard: u32,
        deploy_cost: BigUint<StaticApi>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        self.deploy_and_setup_common(
            deploy_cost.clone(),
            optional_sov_config,
            optional_esdt_safe_config,
            fee,
            Some(shard),
        )
        .await;
    }

    async fn deploy_and_setup_common(
        &mut self,
        deploy_cost: BigUint<StaticApi>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
        target_shard: Option<u32>, // None = all shards, Some(shard) = specific shard
    ) {
        let initial_caller = match target_shard {
            Some(shard) => self.get_bridge_owner_for_shard(shard).clone(),
            None => self.get_bridge_owner_for_shard(SHARD_0).clone(),
        };

        let sovereign_forge_address = self
            .deploy_sovereign_forge(initial_caller.clone(), &BigUint::from(DEPLOY_COST))
            .await;

        for shard_id in 0..NUMBER_OF_SHARDS {
            let caller = self.get_bridge_owner_for_shard(shard_id);
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

        match target_shard {
            Some(shard) => {
                self.deploy_on_one_shard(
                    shard,
                    deploy_cost.clone(),
                    optional_esdt_safe_config.clone(),
                    optional_sov_config.clone(),
                    fee.clone(),
                )
                .await;
            }
            None => {
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

        self.deploy_token_handler(caller.clone(), shard_id).await;
        self.register_token_handler(initial_caller.clone(), shard_id)
            .await;
    }

    async fn deploy_on_one_shard(
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
    }

    async fn register_token_handler(&mut self, caller: Address, shard_id: u32) {
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();
        let token_handler_address = self.state().get_token_handler_address(shard_id).clone();
        let response = self
            .interactor()
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

    async fn register_chain_factory(&mut self, caller: Address, shard_id: u32) {
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();
        let chain_factory_address = self.state().get_chain_factory_sc_address(shard_id).clone();

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

    async fn update_smart_contracts_addresses_in_state(&mut self, chain_id: String) {
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

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
                    self.state().set_chain_config_sc_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                ScArray::ESDTSafe => {
                    self.state()
                        .set_mvx_esdt_safe_contract_address(AddressInfo {
                            address,
                            chain_id: chain_id.clone(),
                        });
                }
                ScArray::FeeMarket => {
                    self.state().set_fee_market_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                ScArray::HeaderVerifier => {
                    self.state().set_header_verifier_address(AddressInfo {
                        address,
                        chain_id: chain_id.clone(),
                    });
                }
                _ => {}
            }
        }
    }

    fn get_contract_info_struct_for_sc_type(
        &mut self,
        sc_array: Vec<ScArray>,
    ) -> Vec<ContractInfo<StaticApi>> {
        sc_array
            .iter()
            .map(|sc| ContractInfo::new(sc.clone(), self.get_sc_address(sc.clone())))
            .collect()
    }

    fn get_sc_address(&mut self, sc_type: ScArray) -> ManagedAddress<StaticApi> {
        match sc_type {
            ScArray::ChainConfig => ManagedAddress::from_address(
                &self.state().current_chain_config_sc_address().to_address(),
            ),
            ScArray::ChainFactory => ManagedAddress::from_address(
                &self.state().current_chain_factory_sc_address().to_address(),
            ),
            ScArray::ESDTSafe => ManagedAddress::from_address(
                &self
                    .state()
                    .current_mvx_esdt_safe_contract_address()
                    .to_address(),
            ),
            ScArray::HeaderVerifier => ManagedAddress::from_address(
                &self.state().current_header_verifier_address().to_address(),
            ),
            ScArray::FeeMarket => ManagedAddress::from_address(
                &self.state().current_fee_market_address().to_address(),
            ),
            ScArray::EnshrineESDTSafe => ManagedAddress::from_address(
                &self
                    .state()
                    .current_enshrine_esdt_safe_address()
                    .to_address(),
            ),
            _ => TestSCAddress::new("ERROR").to_managed_address(),
        }
    }

    async fn deploy_phase_one(
        &mut self,
        caller: Address,
        egld_amount: BigUint<StaticApi>,
        opt_preferred_chain_id: Option<ManagedBuffer<StaticApi>>,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
    ) {
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

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
        caller: Address,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) {
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();
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
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

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
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

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
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

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
        let current_mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();

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
        let current_fee_market_address = self.state().get_fee_market_address(shard).clone();

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
        base_token: TokenIdentifier<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_fee_market_address = self.state().get_fee_market_address(shard).clone();

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
            .state()
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
        let header_verifier_address = self.state().get_header_verifier_address(shard).clone();

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(header_verifier_address)
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

    async fn complete_header_verifier_setup_phase(&mut self, caller: Address) {
        let header_verifier_address = self.state().current_header_verifier_address().clone();

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
        let current_mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();
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
        expected_token: TokenIdentifier<StaticApi>,
        nonce: u64,
        amount: BigUint<StaticApi>,
    ) {
        let user_address = self.user_address().clone();
        let testing_sc_address = self.state().current_testing_sc_address().clone();
        self.interactor()
            .tx()
            .from(user_address)
            .to(testing_sc_address)
            .gas(90_000_000u64)
            .typed(TestingScProxy)
            .send_tokens(
                TokenIdentifier::from_esdt_bytes(expected_token.to_string()),
                nonce,
                amount.clone(),
            )
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
        let current_mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();
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
        args: RegisterTokenArgs<'_>,
        egld_amount: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let user_address = self.user_address().clone();
        let mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();
        let response = self
            .interactor()
            .tx()
            .from(user_address)
            .to(mvx_esdt_safe_address)
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

    async fn get_sov_to_mvx_token_id(
        &mut self,
        shard: u32,
        token_id: TokenIdentifier<StaticApi>,
    ) -> TokenIdentifier<StaticApi> {
        let mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();
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
        token_id: TokenIdentifier<StaticApi>,
        nonce: u64,
    ) -> EsdtInfo<StaticApi> {
        let mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();
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

    async fn whitelist_enshrine_esdt(
        &mut self,
        caller: Address,
        enshrine_esdt_safe_address: Bech32Address,
    ) {
        let token_handler_address = self.state().current_token_handler_address().clone();

        let response = self
            .interactor()
            .tx()
            .from(caller)
            .to(token_handler_address)
            .gas(50_000_000u64)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .whitelist_enshrine_esdt(enshrine_esdt_safe_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    //NOTE: transferValue returns an empty log and calling this function on it will panic
    fn assert_expected_log(
        &mut self,
        logs: Vec<Log>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        match expected_log {
            None => {
                assert!(
                    logs.is_empty(),
                    "Expected no logs, but found some: {:?}",
                    logs
                );
                assert!(
                    expected_log_error.is_none(),
                    "Expected no logs, but wanted to check for error: {}",
                    expected_log_error.unwrap()
                );
            }
            Some(expected_log) => {
                let expected_bytes = expected_log.as_bytes();

                let found_log = logs.iter().find(|log| {
                    log.topics.iter().any(|topic| {
                        if let Ok(decoded_topic) = BASE64.decode(topic) {
                            decoded_topic == expected_bytes
                        } else {
                            false
                        }
                    })
                });

                assert!(
                    found_log.is_some(),
                    "Expected log '{}' not found",
                    expected_log
                );

                if let Some(expected_error) = expected_log_error {
                    let found_log = found_log.unwrap();
                    let expected_error_bytes = expected_error.as_bytes();

                    let found_error_in_data = found_log.data.iter().any(|data_item| {
                        if let Ok(decoded_data) = BASE64.decode(data_item) {
                            decoded_data == expected_error_bytes
                        } else {
                            false
                        }
                    });

                    assert!(
                        found_error_in_data,
                        "Expected error '{}' not found in data field of log with topic '{}'",
                        expected_error, expected_log
                    );
                }
            }
        }
    }

    fn assert_expected_error_message(
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

    /// Key and value should be in hex
    async fn check_account_storage(
        &mut self,
        address: Address,
        wanted_key: &str,
        expected_value: Option<&str>,
    ) {
        let pairs = self.interactor().get_account_storage(&address).await;

        let found_entry = pairs.iter().find(|(key, _)| key.contains(wanted_key));

        let decoded_key = self.decode_from_hex(wanted_key);

        match expected_value {
            Some(expected) => {
                assert!(
                    found_entry.is_some(),
                    "Expected key containing '{}' (decoded: '{}') was not found in account storage.",
                    wanted_key,
                    decoded_key
                );

                let (_, value) = found_entry.unwrap();

                let decoded_expected = self.decode_from_hex(expected);

                let decoded_value = self.decode_from_hex(value);

                assert!(
                    value.contains(expected),
                    "Mismatch: expected '{}' (decoded: '{}') to be contained in '{}' (decoded: '{}')",
                    expected,
                    decoded_expected,
                    value,
                    decoded_value,
                );
            }
            None => {
                assert!(
                    found_entry.is_none(),
                    "Did not expect to find key containing '{}' (decoded: '{}') in account storage.",
                    wanted_key,
                    decoded_key
                );
            }
        }
    }

    async fn check_mvx_esdt_safe_balance_is_empty(&mut self, shard: u32) {
        let mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();

        self.check_address_balance(&mvx_esdt_safe_address, Vec::new())
            .await;
    }

    async fn check_fee_market_balance_is_empty(&mut self, shard: u32) {
        let fee_market_address = self.state().get_fee_market_address(shard).clone();

        self.check_address_balance(&fee_market_address, Vec::new())
            .await;
    }

    async fn check_testing_sc_balance_is_empty(&mut self) {
        let testing_sc_address = self.state().current_testing_sc_address().clone();

        self.check_address_balance(&testing_sc_address, Vec::new())
            .await;
    }

    async fn check_enshrine_esdt_safe_balance_is_empty(&mut self) {
        let enshrine_esdt_safe_address = self.state().current_enshrine_esdt_safe_address().clone();

        self.check_address_balance(&enshrine_esdt_safe_address, Vec::new())
            .await;
    }

    async fn check_address_balance(
        &mut self,
        address: &Bech32Address,
        expected_token_balance: Vec<EsdtTokenInfo>,
    ) {
        let balances = self
            .interactor()
            .get_account_esdt(&address.to_address())
            .await;

        if expected_token_balance.is_empty() {
            assert!(
                balances.is_empty(),
                "Expected no tokens for address {}, but found: {:?}",
                address,
                balances
            );
            return;
        }

        for token_balance in expected_token_balance {
            let token_id = &token_balance.token_id;
            let expected_amount = &token_balance.amount;
            if *expected_amount == 0u64 {
                match balances.get(token_id) {
                    None => {}
                    Some(esdt_balance) => {
                        panic!("For address: {} -> Expected token '{}' to be absent (balance 0), but found it with balance: {}", address, token_id, esdt_balance.balance);
                    }
                }
                continue;
            }

            let complete_tokens = balances.iter().find(|(key, _)| key.starts_with(token_id));

            match complete_tokens {
                Some((found_token_id, esdt_balance)) => {
                    let actual_amount = BigUint::from(
                        num_bigint::BigUint::parse_bytes(esdt_balance.balance.as_bytes(), 10)
                            .expect(FAILED_TO_PARSE_AS_NUMBER),
                    );
                    assert_eq!(
                        actual_amount,
                        *expected_amount,
                        "\nFor address: {} -> Balance mismatch for token {}:\nexpected: {}\nfound:    {}",
                        address,
                        found_token_id,
                        expected_amount.to_display(),
                        esdt_balance.balance
                    );
                }
                None => {
                    panic!(
                        "For address: {} -> Expected token starting with '{}' with balance {}, but none was found",
                        address,
                        token_id,
                        expected_amount.to_display()
                    );
                }
            }
        }
    }

    async fn check_user_balance_unchanged(&mut self) {
        let user_address = self.user_address().clone();
        let expected_balance = self.state().get_initial_wallet_balance().clone().unwrap();

        self.check_address_balance(&Bech32Address::from(user_address), expected_balance)
            .await;
    }

    async fn check_user_balance_after_deduction(
        &mut self,
        token: EsdtTokenInfo,
        deducted_amount: BigUint<StaticApi>,
    ) {
        let expected_balance =
            vec![self.custom_amount_tokens(token.clone(), token.amount.clone() - deducted_amount)];

        self.check_address_balance(
            &Bech32Address::from(self.user_address().clone()),
            expected_balance,
        )
        .await;
    }

    async fn check_user_balance_with_amount(
        &mut self,
        token: EsdtTokenInfo,
        amount: BigUint<StaticApi>,
    ) {
        let expected_balance = vec![self.custom_amount_tokens(token.clone(), amount.clone())];

        self.check_address_balance(
            &Bech32Address::from(self.user_address().clone()),
            expected_balance,
        )
        .await;
    }

    async fn check_user_balance_with_fee_deduction(
        &mut self,
        token: EsdtTokenInfo,
        deducted_amount: BigUint<StaticApi>,
        fee_amount: BigUint<StaticApi>,
    ) {
        let token_balance =
            self.custom_amount_tokens(token.clone(), token.amount.clone() - deducted_amount);
        let fee_token = self.state().get_fee_token_id();
        let fee_balance =
            self.custom_amount_tokens(fee_token.clone(), fee_token.amount.clone() - fee_amount);

        let expected_balances = vec![token_balance, fee_balance];

        self.check_address_balance(
            &Bech32Address::from(self.user_address().clone()),
            expected_balances,
        )
        .await;
    }

    async fn check_mvx_esdt_safe_balance_with_amount(
        &mut self,
        shard: u32,
        token: EsdtTokenInfo,
        amount: BigUint<StaticApi>,
    ) {
        let expected_balance = vec![self.custom_amount_tokens(token.clone(), amount.clone())];
        let mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();

        self.check_address_balance(&mvx_esdt_safe_address, expected_balance)
            .await;
    }

    async fn check_testing_sc_balance_with_amount(
        &mut self,
        token: EsdtTokenInfo,
        amount: BigUint<StaticApi>,
    ) {
        let expected_balance = vec![self.custom_amount_tokens(token.clone(), amount.clone())];
        let testing_sc_address = self.state().current_testing_sc_address().clone();

        self.check_address_balance(&testing_sc_address, expected_balance)
            .await;
    }

    async fn check_fee_market_balance_with_amount(
        &mut self,
        shard: u32,
        token: EsdtTokenInfo,
        amount: BigUint<StaticApi>,
    ) {
        let expected_balance = vec![self.custom_amount_tokens(token.clone(), amount.clone())];
        let fee_market_address = self.state().get_fee_market_address(shard).clone();

        self.check_address_balance(&fee_market_address, expected_balance)
            .await;
    }

    async fn check_setup_phase_status(&mut self, chain_id: &str, expected_value: bool) {
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();
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

    fn get_token_by_type(&mut self, token_type: EsdtTokenType) -> EsdtTokenInfo {
        match token_type {
            EsdtTokenType::NonFungibleV2 => self.state().get_nft_token_id(),
            EsdtTokenType::Fungible => self.state().get_first_token_id(),
            EsdtTokenType::SemiFungible => self.state().get_sft_token_id(),
            EsdtTokenType::MetaFungible => self.state().get_meta_esdt_token_id(),
            EsdtTokenType::DynamicNFT => self.state().get_dynamic_nft_token_id(),
            EsdtTokenType::DynamicSFT => self.state().get_dynamic_sft_token_id(),
            EsdtTokenType::DynamicMeta => self.state().get_dynamic_meta_esdt_token_id(),
            _ => panic!("Unsupported token type for test"),
        }
    }

    fn create_standard_fee(&mut self) -> FeeStruct<StaticApi> {
        let per_transfer = BigUint::from(PER_TRANSFER);
        let per_gas = BigUint::from(PER_GAS);
        FeeStruct {
            base_token: self.state().get_fee_token_identifier(),
            fee_type: FeeType::Fixed {
                token: self.state().get_fee_token_identifier(),
                per_transfer,
                per_gas,
            },
        }
    }

    fn get_bridge_service_for_shard(&self, shard_id: u32) -> Address {
        let shard_0_wallet = Wallet::from_pem_file("wallets/shard-0-wallet.pem")
            .expect(FAILED_TO_LOAD_WALLET_SHARD_0);
        match shard_id {
            0 => shard_0_wallet.to_address(),
            1 => test_wallets::dan().to_address(),
            2 => test_wallets::judy().to_address(),
            _ => panic!("Invalid shard ID: {shard_id}"),
        }
    }
    fn get_bridge_owner_for_shard(&self, shard_id: u32) -> Address {
        match shard_id {
            0 => test_wallets::bob().to_address(),
            1 => test_wallets::alice().to_address(),
            2 => test_wallets::carol().to_address(),
            _ => panic!("Invalid shard ID: {shard_id}"),
        }
    }

    fn get_sovereign_owner_for_shard(&self, shard_id: u32) -> Address {
        match shard_id {
            0 => test_wallets::mike().to_address(),
            1 => test_wallets::frank().to_address(),
            2 => test_wallets::heidi().to_address(),
            _ => panic!("Invalid shard ID: {shard_id}"),
        }
    }

    fn decode_from_hex(&mut self, hex_string: &str) -> String {
        let bytes =
            hex::decode(hex_string).expect("Failed to decode hex string: invalid hex format");
        String::from_utf8(bytes).expect("Failed to decode UTF-8 string: invalid UTF-8 bytes")
    }

    fn get_operation_hash(&mut self, operation: &Operation<StaticApi>) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }

    fn custom_amount_tokens(
        &mut self,
        token: EsdtTokenInfo,
        new_amount: BigUint<StaticApi>,
    ) -> EsdtTokenInfo {
        EsdtTokenInfo {
            token_id: token.token_id,
            amount: new_amount,
            nonce: token.nonce,
            token_type: token.token_type,
        }
    }
}
