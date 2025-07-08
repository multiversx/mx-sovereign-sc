#![allow(async_fn_in_trait)]

use crate::interactor_state::{AddressInfo, State, TokenProperties};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use common_test_setup::constants::{
    CHAIN_CONFIG_CODE_PATH, CHAIN_FACTORY_CODE_PATH, ENSHRINE_ESDT_SAFE_CODE_PATH,
    FEE_MARKET_CODE_PATH, HEADER_VERIFIER_CODE_PATH, ISSUE_COST, MVX_ESDT_SAFE_CODE_PATH,
    ONE_HUNDRED_TOKENS, ONE_THOUSAND_TOKENS, SOVEREIGN_FORGE_CODE_PATH, SOVEREIGN_TOKEN_PREFIX,
    TESTING_SC_CODE_PATH, TOKEN_HANDLER_CODE_PATH, WEGLD_IDENTIFIER,
};
use error_messages::{EMPTY_EXPECTED_LOG, FAILED_TO_PARSE_AS_NUMBER};
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
        bech32, Bech32Address, ReturnsGasUsed, ReturnsHandledOrError, ReturnsLogs,
        ReturnsNewTokenIdentifier, StaticApi,
    },
    multiversx_sc_scenario::{
        multiversx_chain_vm::crypto_functions::sha256,
        scenario_model::{Log, TxResponseStatus},
    },
    Interactor, InteractorRunAsync,
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
    fee::FeeStruct,
    forge::{ContractInfo, ScArray},
    operation::Operation,
};

use common_test_setup::base_setup::init::RegisterTokenArgs;

pub struct IssueTokenStruct {
    pub token_display_name: String,
    pub token_ticker: String,
    pub token_type: EsdtTokenType,
    pub num_decimals: usize,
}

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
    fn sovereign_owner(&self) -> &Address;
    fn bridge_service(&self) -> &Address;
    fn user_address(&self) -> &Address;

    async fn issue_and_mint_token(
        &mut self,
        issue: IssueTokenStruct,
        mint: MintTokenStruct,
    ) -> TokenProperties {
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
            .mint_tokens(token_id.clone(), issue.token_type, mint)
            .await;

        TokenProperties {
            token_id: token_id.clone(),
            nonce,
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
            EsdtTokenType::NonFungible
            | EsdtTokenType::SemiFungible
            | EsdtTokenType::DynamicNFT
            | EsdtTokenType::DynamicMeta
            | EsdtTokenType::DynamicSFT
            | EsdtTokenType::Meta => {
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
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state()
            .set_sovereign_forge_sc_address(new_address_bech32);

        new_address
    }

    async fn deploy_chain_factory(
        &mut self,
        caller: Address,
        chain_id: String,
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
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state().set_chain_factory_sc_address(AddressInfo {
            address: new_address_bech32.clone(),
            chain_id,
        });
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
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
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
            .code_metadata(CodeMetadata::all())
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
                    .code_metadata(CodeMetadata::all())
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
                    .code_metadata(CodeMetadata::all())
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
            .code_metadata(CodeMetadata::all())
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
            .code_metadata(CodeMetadata::all())
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
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
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
    ) -> Bech32Address {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(100_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(opt_config)
            .returns(ReturnsNewAddress)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state()
            .set_mvx_esdt_safe_contract_address(AddressInfo {
                address: new_address_bech32.clone(),
                chain_id,
            });

        new_address_bech32
    }

    async fn deploy_fee_market(
        &mut self,
        caller: Address,
        chain_id: String,
        esdt_safe_address: Bech32Address,
        fee: Option<FeeStruct<StaticApi>>,
    ) -> Bech32Address {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(80_000_000u64)
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .returns(ReturnsNewAddress)
            .code(FEE_MARKET_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state().set_fee_market_address(AddressInfo {
            address: new_address_bech32.clone(),
            chain_id,
        });
        new_address_bech32
    }

    async fn deploy_testing_sc(&mut self, caller: Address, chain_id: String) -> Bech32Address {
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(120_000_000u64)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state().set_testing_sc_address(AddressInfo {
            address: new_address_bech32.clone(),
            chain_id,
        });

        new_address_bech32
    }

    async fn deploy_token_handler(&mut self, caller: Address, chain_id: String) {
        let chain_factory_address = self
            .state()
            .get_chain_factory_sc_address(chain_id.clone())
            .clone();

        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(100_000_000u64)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init(chain_factory_address)
            .code(TOKEN_HANDLER_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state().set_token_handler_address(AddressInfo {
            address: new_address_bech32,
            chain_id,
        });
    }

    //TODO: Remove this after enshrine changes
    async fn deploy_token_handler_enshrine(&mut self, caller: Address, chain_id: String) {
        let new_address = self
            .interactor()
            .tx()
            .from(caller.clone())
            .gas(100_000_000u64)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init(caller)
            .code(TOKEN_HANDLER_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state().set_token_handler_address(AddressInfo {
            address: new_address_bech32,
            chain_id,
        });
    }

    async fn deploy_enshrine_esdt(
        &mut self,
        caller: Address,
        chain_id: String,
        is_sovereign_chain: bool,
        opt_wegld_identifier: Option<TokenIdentifier<StaticApi>>,
        opt_sov_token_prefix: Option<ManagedBuffer<StaticApi>>,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
    ) {
        let token_handler_address = self
            .state()
            .get_token_handler_address(chain_id.clone())
            .clone();
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
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = Bech32Address::from_bech32_string(bech32::encode(&new_address));
        self.state().set_enshrine_esdt_safe_sc_address(AddressInfo {
            address: new_address_bech32,
            chain_id,
        });
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

    async fn change_ownership_to_header_verifier(
        &mut self,
        initial_owner: Address,
        sc_address: Address,
    ) {
        let managed_header_verifier_address = ManagedAddress::from_address(
            self.state().current_header_verifier_address().as_address(),
        );

        self.interactor()
            .tx()
            .from(initial_owner)
            .to(sc_address)
            .gas(90_000_000u64)
            .typed(UserBuiltinProxy)
            .change_owner_address(&managed_header_verifier_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn update_esdt_safe_config(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        new_config: EsdtSafeConfig<StaticApi>,
    ) {
        let bridge_service = self.bridge_service().clone();
        let current_mvx_esdt_safe_address = self
            .state()
            .current_mvx_esdt_safe_contract_address()
            .clone();

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
    ) {
        let bridge_service = self.bridge_service().clone();
        let current_fee_market_address = self.state().current_fee_market_address().clone();

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
    ) {
        let bridge_service = self.bridge_service().clone();
        let current_fee_market_address = self.state().current_fee_market_address().clone();

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
        let sovereign_owner = self.sovereign_owner().clone();

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
        let bridge_service = self.bridge_service().clone();
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

        println!("Logs: {:?}", logs);

        self.assert_expected_log(logs, expected_log);
    }

    async fn deposit_in_mvx_esdt_safe_test(
        &mut self,
        to: Address,
        mvx_address: Bech32Address,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payments: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let user_address = self.user_address().clone();
        let (response, logs) = self
            .interactor()
            .tx()
            .from(user_address)
            .to(mvx_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        println!("Logs: {:?}", logs);

        self.assert_expected_log(logs, expected_log);
    }

    async fn execute_operations_in_mvx_esdt_safe(
        &mut self,
        shard: u32,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let bridge_service = self.bridge_service().clone();
        let current_mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();
        let (response, logs) = self
            .interactor()
            .tx()
            .from(bridge_service)
            .to(current_mvx_esdt_safe_address)
            .gas(120_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        self.assert_expected_log(logs, expected_log);
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
    //TODO: cross shard transactions do not return the same type of logs like the ones on the same shard
    fn assert_expected_log(&mut self, logs: Vec<Log>, expected_log: Option<&str>) {
        match expected_log {
            None => {
                assert!(
                    logs.is_empty(),
                    "Expected no logs, but found some: {:?}",
                    logs
                );
            }
            Some(expected_log) => {
                assert!(!expected_log.is_empty(), "{}", EMPTY_EXPECTED_LOG);
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

    async fn check_wallet_balance_unchanged(&mut self) {
        let user_address = self.user_address().clone();
        let first_token_id = self.state().get_first_token_id_string();
        let second_token_id = self.state().get_second_token_id_string();
        let fee_token_id = self.state().get_fee_token_id_string();
        let nft_token_id = self.state().get_nft_token_id_string();
        let sft_token_id = self.state().get_sft_token_id_string();
        let meta_esdt_token_id = self.state().get_meta_esdt_token_id_string();
        let dynamic_nft_token_id = self.state().get_dynamic_nft_token_id_string();

        let expected_tokens_wallet = vec![
            self.thousand_tokens(first_token_id),
            self.thousand_tokens(second_token_id),
            self.thousand_tokens(fee_token_id),
            self.one_token(nft_token_id),
            self.thousand_tokens(sft_token_id),
            self.thousand_tokens(meta_esdt_token_id),
            self.one_token(dynamic_nft_token_id),
        ];

        self.check_address_balance(&Bech32Address::from(user_address), expected_tokens_wallet)
            .await;
    }

    async fn check_mvx_esdt_safe_balance_is_empty(&mut self, shard: u32) {
        let first_token_id = self.state().get_first_token_id_string();
        let second_token_id = self.state().get_second_token_id_string();
        let mvx_esdt_safe_address = self.state().get_mvx_esdt_safe_address(shard).clone();

        let expected_tokens_mvx_esdt_safe = vec![
            self.zero_tokens(first_token_id),
            self.zero_tokens(second_token_id),
        ];

        self.check_address_balance(&mvx_esdt_safe_address, expected_tokens_mvx_esdt_safe)
            .await;
    }

    async fn check_fee_market_balance_is_empty(&mut self, shard: u32) {
        let fee_market_address = self.state().get_fee_market_address(shard).clone();
        let fee_token_id = self.state().get_fee_token_id_string();

        let expected_tokens_fee_market = vec![self.zero_tokens(fee_token_id)];

        self.check_address_balance(&fee_market_address, expected_tokens_fee_market)
            .await;
    }

    async fn check_testing_sc_balance_is_empty(&mut self, shard: u32) {
        let testing_sc_address = self.state().get_testing_sc_address(shard).clone();
        let first_token_id = self.state().get_first_token_id_string();

        let expected_tokens_testing_sc = vec![self.zero_tokens(first_token_id)];

        self.check_address_balance(&testing_sc_address, expected_tokens_testing_sc)
            .await;
    }

    async fn check_enshrine_esdt_safe_balance_is_empty(&mut self) {
        let enshrine_esdt_safe_address = self.state().current_enshrine_esdt_safe_address().clone();
        let first_token_id = self.state().get_first_token_id_string();
        let second_token_id = self.state().get_second_token_id_string();

        let expected_tokens_enshrine_esdt_safe = vec![
            self.zero_tokens(first_token_id),
            self.zero_tokens(second_token_id),
        ];

        self.check_address_balance(
            &enshrine_esdt_safe_address,
            expected_tokens_enshrine_esdt_safe,
        )
        .await;
    }

    async fn check_address_balance(
        &mut self,
        address: &Bech32Address,
        expected_tokens: Vec<(String, BigUint<StaticApi>)>,
    ) {
        let balances = self
            .interactor()
            .get_account_esdt(&address.to_address())
            .await;

        for (token_id, expected_amount) in expected_tokens {
            if expected_amount == 0u64 {
                match balances.get(&token_id) {
                    None => {}
                    Some(esdt_balance) => {
                        panic!("Expected token '{}' to be absent (balance 0), but found it with balance: {}", token_id, esdt_balance.balance);
                    }
                }
                continue;
            }
            let complete_tokens = balances.iter().find(|(key, _)| key.starts_with(&token_id));

            match complete_tokens {
                Some((token_id, esdt_balance)) => {
                    if expected_amount == 0u64 {
                        panic!(
                        "Expected token starting with '{}' to be absent, but found: {} with balance: {}",
                        token_id, token_id, esdt_balance.balance);
                    }

                    let actual_amount = BigUint::from(
                        num_bigint::BigUint::parse_bytes(esdt_balance.balance.as_bytes(), 10)
                            .expect(FAILED_TO_PARSE_AS_NUMBER),
                    );

                    assert_eq!(
                        actual_amount,
                        expected_amount,
                        "\nBalance mismatch for token {}:\nexpected: {}\nfound:    {}",
                        token_id,
                        expected_amount.to_display(),
                        esdt_balance.balance
                    );
                }
                None => {
                    panic!(
                        "Expected token starting with '{}' with balance {}, but none was found",
                        token_id,
                        expected_amount.to_display()
                    );
                }
            }
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

    fn thousand_tokens(&mut self, token_id: String) -> (String, BigUint<StaticApi>) {
        (token_id, BigUint::from(ONE_THOUSAND_TOKENS))
    }

    fn hundred_tokens(&mut self, token_id: String) -> (String, BigUint<StaticApi>) {
        (token_id, BigUint::from(ONE_HUNDRED_TOKENS))
    }

    fn zero_tokens(&mut self, token_id: String) -> (String, BigUint<StaticApi>) {
        (token_id, BigUint::from(0u64))
    }

    fn one_token(&mut self, token_id: String) -> (String, BigUint<StaticApi>) {
        (token_id, BigUint::from(1u64))
    }

    fn custom_amount_tokens<T: Into<BigUint<StaticApi>>>(
        &mut self,
        token_id: impl Into<String>,
        amount: T,
    ) -> (String, BigUint<StaticApi>) {
        (token_id.into(), amount.into())
    }
}
