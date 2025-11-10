#![allow(async_fn_in_trait)]
use crate::{
    interactor_helpers::InteractorHelpers,
    interactor_state::{AddressInfo, EsdtTokenInfo},
    interactor_structs::{ActionConfig, IssueTokenStruct, MintTokenStruct, TemplateAddresses},
};
use common_test_setup::constants::{
    CHAIN_CONFIG_CODE_PATH, CHAIN_FACTORY_CODE_PATH, CHAIN_ID, DEPLOY_COST,
    FAILED_TO_LOAD_WALLET_SHARD_0, FEE_MARKET_CODE_PATH, HEADER_VERIFIER_CODE_PATH, ISSUE_COST,
    MVX_ESDT_SAFE_CODE_PATH, NATIVE_TOKEN_NAME, NATIVE_TOKEN_TICKER, NUMBER_OF_SHARDS,
    NUM_TOKENS_TO_MINT, ONE_THOUSAND_TOKENS, SHARD_0, SOVEREIGN_FORGE_CODE_PATH,
    SOVEREIGN_TOKEN_PREFIX, TESTING_SC_CODE_PATH, WALLET_SHARD_0,
};
use multiversx_bls::{SecretKey, G1};
use multiversx_sc::{
    codec::num_bigint,
    imports::{ESDTSystemSCProxy, OptionalValue, UserBuiltinProxy},
    types::{
        Address, BigUint, CodeMetadata, ESDTSystemSCAddress, EgldOrEsdtTokenIdentifier,
        EsdtLocalRole, EsdtTokenType, ManagedAddress, ManagedBuffer, ManagedVec,
        MultiEgldOrEsdtPayment, MultiValueEncoded, ReturnsNewAddress, ReturnsResult,
        ReturnsResultUnmanaged, TokenIdentifier,
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
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
    header_verifier_proxy::HeaderverifierProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    mvx_fee_market_proxy::MvxFeeMarketProxy, sovereign_forge_proxy::SovereignForgeProxy,
    testing_sc_proxy::TestingScProxy,
};
use structs::{
    aliases::{OptionalValueTransferDataTuple, PaymentsVec, TxNonce},
    configs::{
        EsdtSafeConfig, PauseStatusOperation, SetBurnMechanismOperation, SetLockMechanismOperation,
        SovereignConfig, UpdateEsdtSafeConfigOperation,
    },
    fee::{FeeStruct, RemoveFeeOperation, SetFeeOperation},
    forge::{ContractInfo, ScArray},
    generate_hash::GenerateHash,
    operation::Operation,
    EsdtInfo, OperationHashStatus, RegisterTokenOperation,
};

fn metadata() -> CodeMetadata {
    CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE
}

pub trait CommonInteractorTrait: InteractorHelpers {
    async fn register_wallets(&mut self) {
        let wallet_path = WALLET_SHARD_0.to_string();
        let wallet = Wallet::from_pem_file(&wallet_path)
            .unwrap_or_else(|_| panic!("{}", FAILED_TO_LOAD_WALLET_SHARD_0));

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

    async fn issue_and_mint_token(&mut self, issue: IssueTokenStruct, mint: MintTokenStruct) {
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
                issue.token_ticker.clone(),
                issue.token_type,
                issue.num_decimals,
            )
            .returns(ReturnsNewTokenIdentifier)
            .run()
            .await;

        let num_mints = if issue.token_ticker == "TRUSTED" || issue.token_ticker == "FEE" {
            1
        } else {
            NUM_TOKENS_TO_MINT
        };

        for _ in 0..num_mints {
            let nonce = self
                .mint_tokens(token_id.clone(), issue.token_type, mint.clone())
                .await;

            let decimals = self.get_token_decimals(issue.token_type);

            let token = EsdtTokenInfo {
                token_id: EgldOrEsdtTokenIdentifier::from(token_id.as_bytes()),
                nonce,
                token_type: issue.token_type,
                decimals,
                amount: mint.amount.clone(),
            };

            match issue.token_ticker.as_str() {
                "MVX" => self.state().add_fungible_token(token.clone()),
                "TRUSTED" => self.common_state().set_trusted_token(token_id.clone()),
                "FEE" => self.state().set_fee_token(token.clone()),
                "NFT" => self.state().add_nft_token(token.clone()),
                "SFT" => self.state().add_sft_token(token.clone()),
                "DYN" => self.state().add_dynamic_nft_token(token.clone()),
                "META" => self.state().add_meta_esdt_token(token.clone()),
                "DYNS" => self.state().add_dynamic_sft_token(token.clone()),
                "DYNM" => self.state().add_dynamic_meta_esdt_token(token.clone()),
                _ => {}
            }

            self.state()
                .update_or_add_initial_wallet_token(token.clone());
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
        decimals: usize,
    ) {
        if ticker == "FEE" && !self.common_state().fee_market_tokens.is_empty() {
            let fee_token = self.retrieve_current_fee_token_for_wallet().await;
            self.state().set_fee_token(fee_token);
            return;
        }
        if ticker == "TRUSTED" && self.common_state().trusted_token.is_some() {
            let trusted_token = self.retrieve_current_trusted_token_for_wallet().await;
            self.state().set_trusted_token(trusted_token.clone());
            self.state()
                .update_or_add_initial_wallet_token(trusted_token.clone());
            return;
        }
        let amount = if matches!(
            token_type,
            EsdtTokenType::NonFungibleV2 | EsdtTokenType::DynamicNFT
        ) {
            BigUint::from(1u64)
        } else {
            BigUint::from(ONE_THOUSAND_TOKENS)
        };
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

        self.issue_and_mint_token(token_struct, mint_struct).await;
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
        forge_address: &Address,
        chain_id: &str,
    ) -> Vec<Bech32Address> {
        let mut template_contracts = vec![];

        let chain_config_template = self
            .interactor()
            .tx()
            .from(caller.clone())
            .gas(50_000_000u64)
            .typed(ChainConfigContractProxy)
            .init(OptionalValue::<SovereignConfig<StaticApi>>::Some(
                SovereignConfig::default_config_for_test(),
            ))
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
            .gas(120_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(
                Bech32Address::from(caller.clone()),
                forge_address,
                chain_id,
                OptionalValue::<EsdtSafeConfig<StaticApi>>::None,
            )
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
            .typed(MvxFeeMarketProxy)
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
        forge_address: &Address,
        chain_id: String,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) {
        let owner_address = caller.clone();
        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(100_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(
                owner_address,
                forge_address,
                SOVEREIGN_TOKEN_PREFIX,
                opt_config,
            )
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
        payment: MultiEgldOrEsdtPayment<StaticApi>,
        chain_config_address: Bech32Address,
    ) {
        let bridge_owner = self.get_bridge_owner_for_shard(shard).clone();

        let mut secret_key = SecretKey::default();
        secret_key.set_by_csprng();

        let secret_key_bytes = secret_key
            .serialize()
            .expect("Failed to serialize BLS secret key");
        let public_key_bytes = secret_key
            .get_public_key()
            .serialize()
            .expect("Failed to serialize BLS public key");

        self.common_state()
            .add_bls_secret_key(shard, secret_key_bytes);

        let bls_key_buffer = ManagedBuffer::<StaticApi>::new_from_bytes(&public_key_bytes);

        self.interactor()
            .tx()
            .from(bridge_owner)
            .to(chain_config_address)
            .gas(90_000_000u64)
            .typed(ChainConfigContractProxy)
            .register(bls_key_buffer)
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
        fee: OptionalValue<FeeStruct<StaticApi>>,
    ) {
        let fee = fee.into_option();

        let new_address = self
            .interactor()
            .tx()
            .from(caller)
            .gas(80_000_000u64)
            .typed(MvxFeeMarketProxy)
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
            OptionalValue::Some(fee_struct),
        )
        .await;
        let fee_token_id = self.state().get_fee_token_id();
        let fee_token_fee_market = self.create_serializable_token(fee_token_id, 0u64);
        self.common_state()
            .set_fee_market_token_for_all_shards(fee_token_fee_market);
        self.common_state().set_fee_status_for_all_shards(true);
        self.common_state()
            .set_mvx_egld_balance_for_all_shards(0u64);
    }

    async fn unpause_forge(&mut self, sovereign_forge_address: Address) {
        let bridge_owner = self.get_bridge_owner_for_shard(SHARD_0).clone();

        self.interactor()
            .tx()
            .from(bridge_owner)
            .to(sovereign_forge_address)
            .typed(SovereignForgeProxy)
            .unpause_endpoint()
            .gas(20_000_000)
            .run()
            .await;
    }

    async fn deploy_and_setup_common(
        &mut self,
        deploy_cost: OptionalValue<BigUint<StaticApi>>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee: OptionalValue<FeeStruct<StaticApi>>,
    ) {
        let initial_caller = self.get_bridge_owner_for_shard(SHARD_0).clone();

        let sovereign_forge_address = self
            .deploy_sovereign_forge(
                initial_caller.clone(),
                OptionalValue::Some(&BigUint::from(DEPLOY_COST)),
            )
            .await;
        self.unpause_forge(sovereign_forge_address.clone()).await;

        let trusted_token = self.common_state().get_trusted_token();

        self.register_trusted_token(initial_caller.clone(), trusted_token.as_str())
            .await;

        for shard_id in 0..NUMBER_OF_SHARDS {
            let caller = self.get_bridge_owner_for_shard(shard_id);
            let template_contracts = self
                .deploy_template_contracts(caller.clone(), &sovereign_forge_address, CHAIN_ID)
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

    async fn register_trusted_token(&mut self, caller: Address, trusted_token: &str) {
        let forge_address = &self
            .common_state()
            .sovereign_forge_sc_address
            .clone()
            .unwrap();

        self.interactor()
            .tx()
            .from(caller)
            .to(forge_address)
            .typed(SovereignForgeProxy)
            .register_trusted_token(ManagedBuffer::from(trusted_token))
            .gas(20_000_000)
            .run()
            .await;
    }

    async fn deploy_on_one_shard(
        &mut self,
        shard: u32,
        deploy_cost: OptionalValue<BigUint<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        fee: OptionalValue<FeeStruct<StaticApi>>,
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
            MultiEgldOrEsdtPayment::new(),
            chain_config_address.clone(),
        )
        .await;

        self.deploy_phase_two(optional_esdt_safe_config.clone(), caller.clone())
            .await;
        self.register_native_token(caller.clone(), &preferred_chain_id)
            .await;

        let mvx_esdt_safe_address = self
            .get_sc_address_from_sovereign_forge(preferred_chain_id.as_str(), ScArray::ESDTSafe)
            .await;

        self.set_special_roles_for_trusted_token(mvx_esdt_safe_address.clone())
            .await;

        self.deploy_phase_three(caller.clone(), fee.clone()).await;
        self.deploy_phase_four(caller.clone()).await;

        self.complete_setup_phase(caller.clone()).await;
        self.check_setup_phase_status(&preferred_chain_id, true)
            .await;

        self.update_smart_contracts_addresses_in_state(preferred_chain_id.clone())
            .await;

        println!("Finished deployment for shard {shard}");
    }

    async fn register_native_token(&mut self, caller: Address, chain_id: &str) {
        let mvx_esdt_safe_address = self
            .get_sc_address_from_sovereign_forge(chain_id, ScArray::ESDTSafe)
            .await;

        self.interactor()
            .tx()
            .from(caller)
            .to(mvx_esdt_safe_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .register_native_token(
                ManagedBuffer::from(NATIVE_TOKEN_TICKER),
                ManagedBuffer::from(NATIVE_TOKEN_NAME),
            )
            .egld(BigUint::from(ISSUE_COST))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn get_sc_address_from_sovereign_forge(
        &mut self,
        chain_id: &str,
        sc_id: ScArray,
    ) -> Address {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        self.interactor()
            .query()
            .to(sovereign_forge_address)
            .typed(SovereignForgeProxy)
            .sovereign_deployed_contracts(chain_id)
            .returns(ReturnsResult)
            .run()
            .await
            .into_iter()
            .find(|sc| sc.id == sc_id)
            .unwrap()
            .address
            .to_address()
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

        self.interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, chain_factory_address)
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

        self.interactor()
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
        self.interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_two(opt_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn deploy_phase_three(
        &mut self,
        caller: Address,
        fee: OptionalValue<FeeStruct<StaticApi>>,
    ) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        self.interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_three(fee)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn deploy_phase_four(&mut self, caller: Address) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();

        self.interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_four()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
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
        esdt_safe_config: EsdtSafeConfig<StaticApi>,
        nonce: TxNonce,
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
            .update_esdt_safe_config(
                hash_of_hashes,
                UpdateEsdtSafeConfigOperation {
                    esdt_safe_config,
                    nonce,
                },
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn set_fee(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        fee_operation: SetFeeOperation<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_fee_market_address = self.common_state().get_fee_market_address(shard).clone();

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(current_fee_market_address)
            .gas(50_000_000u64)
            .typed(MvxFeeMarketProxy)
            .set_fee(hash_of_hashes, fee_operation)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn remove_fee(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        fee_operation: RemoveFeeOperation<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_fee_market_address = self.common_state().get_fee_market_address(shard).clone();

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(current_fee_market_address)
            .gas(90_000_000u64)
            .typed(MvxFeeMarketProxy)
            .remove_fee(hash_of_hashes, fee_operation)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn set_token_burn_mechanism(
        &mut self,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_mvx_esdt_safe_address =
            self.common_state().get_mvx_esdt_safe_address(shard).clone();

        if self.common_state().get_is_burn_mechanism_set() {
            return;
        }

        let token_burn_mechanism_operation = SetBurnMechanismOperation {
            token_id,
            nonce: self
                .common_state()
                .get_and_increment_operation_nonce(&current_mvx_esdt_safe_address.to_string()),
        };

        let token_burn_mechanism_operation_hash = token_burn_mechanism_operation.generate_hash();
        let token_burn_mechanism_hash_of_hashes =
            ManagedBuffer::new_from_bytes(&sha256(&token_burn_mechanism_operation_hash.to_vec()));

        self.register_operation(
            shard,
            &token_burn_mechanism_hash_of_hashes,
            MultiValueEncoded::from(ManagedVec::from(vec![token_burn_mechanism_operation_hash])),
        )
        .await;

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(current_mvx_esdt_safe_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_token_burn_mechanism(
                token_burn_mechanism_hash_of_hashes,
                token_burn_mechanism_operation,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        self.common_state().set_is_burn_mechanism_set(true);
    }

    async fn set_token_lock_mechanism(
        &mut self,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
        shard: u32,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let current_mvx_esdt_safe_address =
            self.common_state().get_mvx_esdt_safe_address(shard).clone();

        let token_lock_mechanism_operation = SetLockMechanismOperation {
            token_id,
            nonce: self
                .common_state()
                .get_and_increment_operation_nonce(&current_mvx_esdt_safe_address.to_string()),
        };

        let token_lock_mechanism_operation_hash = token_lock_mechanism_operation.generate_hash();
        let token_lock_mechanism_hash_of_hashes =
            ManagedBuffer::new_from_bytes(&sha256(&token_lock_mechanism_operation_hash.to_vec()));

        self.register_operation(
            shard,
            &token_lock_mechanism_hash_of_hashes,
            MultiValueEncoded::from(ManagedVec::from(vec![token_lock_mechanism_operation_hash])),
        )
        .await;

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(current_mvx_esdt_safe_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_token_lock_mechanism(
                token_lock_mechanism_hash_of_hashes,
                token_lock_mechanism_operation,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        self.common_state().set_is_burn_mechanism_set(false);
    }

    async fn set_token_burn_mechanism_before_setup_phase(&mut self, caller: Address) {
        let sovereign_forge_address = self
            .common_state()
            .current_sovereign_forge_sc_address()
            .clone();
        let trusted_token = self.common_state().get_trusted_token();

        self.interactor()
            .tx()
            .from(caller)
            .to(sovereign_forge_address)
            .gas(90_000_000u64)
            .typed(SovereignForgeProxy)
            .set_token_burn_mechanism(EgldOrEsdtTokenIdentifier::esdt(trusted_token.as_str()))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn set_special_roles_for_trusted_token(&mut self, for_address: Address) {
        let user_address = self.user_address().clone();
        let trusted_token = self.common_state().get_trusted_token();

        let roles = vec![EsdtLocalRole::Mint, EsdtLocalRole::Burn];

        self.interactor()
            .tx()
            .from(user_address)
            .to(ESDTSystemSCAddress)
            .gas(80_000_000u64)
            .typed(ESDTSystemSCProxy)
            .set_special_roles(
                ManagedAddress::from_address(&for_address),
                TokenIdentifier::from(trusted_token.as_str()),
                roles.into_iter(),
            )
            .run()
            .await;
    }

    async fn register_operation(
        &mut self,
        shard: u32,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard).clone();
        let header_verifier_address = self
            .common_state()
            .get_header_verifier_address(shard)
            .clone();

        let secret_keys = self
            .common_state()
            .get_bls_secret_keys(shard)
            .cloned()
            .unwrap_or_else(|| panic!("No BLS secret keys registered for shard {shard}"));

        let (signature, bitmap) =
            Self::create_aggregated_signature_and_bitmap(&secret_keys, hash_of_hashes);
        let epoch = 0u32;

        self.interactor()
            .tx()
            .from(bridge_service)
            .to(header_verifier_address)
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                signature,
                hash_of_hashes.clone(),
                bitmap,
                epoch,
                operations_hashes,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    fn create_aggregated_signature_and_bitmap(
        secret_keys: &[Vec<u8>],
        message: &ManagedBuffer<StaticApi>,
    ) -> (ManagedBuffer<StaticApi>, ManagedBuffer<StaticApi>) {
        assert!(
            !secret_keys.is_empty(),
            "At least one BLS key is required to build the signature bitmap",
        );

        let message_bytes = message.to_vec();
        let mut signatures = Vec::with_capacity(secret_keys.len());

        for key_bytes in secret_keys {
            let secret_key = SecretKey::from_serialized(key_bytes)
                .expect("Failed to deserialize stored BLS secret key");
            let signature = secret_key.sign(&message_bytes);
            signatures.push(signature);
        }

        let mut aggregated_signature = G1::default();
        aggregated_signature.aggregate(&signatures);

        let signature_bytes = aggregated_signature
            .serialize()
            .expect("Failed to serialize aggregated BLS signature");
        let bitmap_bytes = Self::build_bitmap(secret_keys.len());

        (
            ManagedBuffer::new_from_bytes(&signature_bytes),
            ManagedBuffer::new_from_bytes(&bitmap_bytes),
        )
    }

    fn build_bitmap(num_signers: usize) -> Vec<u8> {
        assert!(num_signers > 0, "Cannot build bitmap with zero signers");

        let byte_len = num_signers.div_ceil(8);
        let mut bitmap = vec![0u8; byte_len];

        for signer_index in 0..num_signers {
            let byte_index = signer_index / 8;
            let bit_index = signer_index % 8;
            bitmap[byte_index] |= 1 << bit_index;
        }

        bitmap
    }

    async fn switch_pause_status(&mut self, status: bool, shard: u32) {
        let mvx_esdt_safe_address = self
            .common_state()
            .current_mvx_esdt_safe_contract_address()
            .clone();
        let bridge_address = self.get_bridge_service_for_shard(shard).clone();

        let operation = PauseStatusOperation {
            status,
            nonce: self
                .common_state()
                .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
        };

        let operation_hash = operation.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));
        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

        self.register_operation(shard, &hash_of_hashes, operations_hashes)
            .await;

        self.interactor()
            .tx()
            .from(bridge_address)
            .to(mvx_esdt_safe_address.clone())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .switch_pause_status(hash_of_hashes, operation)
            .run()
            .await;

        let current_status = self
            .interactor()
            .query()
            .to(mvx_esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResult)
            .run()
            .await;

        assert_eq!(current_status, status, "Pause status is not correct");
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
        nonce: TxNonce,
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
            .gas(130_000_000u64)
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
            .register_sovereign_token(hash_of_hashes, token)
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
        hash_of_hashes: &ManagedBuffer<StaticApi>,
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
            || (self.is_nft(original_token) && config.expected_log_error.is_some());

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

    async fn retrieve_current_trusted_token_for_wallet(&mut self) -> EsdtTokenInfo {
        let user_address = &self.user_address().clone();
        let balances = self.interactor().get_account_esdt(user_address).await;
        let trusted_token = self.common_state().get_trusted_token();

        let amount = if let Some(esdt_balance) = balances.get(trusted_token.as_str()) {
            BigUint::from(
                num_bigint::BigUint::parse_bytes(esdt_balance.balance.as_bytes(), 10)
                    .expect("Failed to parse fee token balance as number"),
            )
        } else {
            BigUint::zero()
        };

        EsdtTokenInfo {
            token_id: EgldOrEsdtTokenIdentifier::esdt(trusted_token.as_str()),
            nonce: 0,
            token_type: EsdtTokenType::Fungible,
            amount,
            decimals: 18,
        }
    }

    async fn remove_fee_wrapper(&mut self, shard: u32) {
        let fee_activated = self.common_state().get_fee_status_for_shard(shard);

        if !fee_activated {
            return;
        }

        let fee_token = self.state().get_fee_token_identifier();
        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();

        let operation: RemoveFeeOperation<StaticApi> = RemoveFeeOperation {
            token_id: fee_token.clone(),
            nonce: self
                .common_state()
                .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
        };

        let operation_hash = operation.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        let operations_hashes = MultiValueEncoded::from_iter(vec![operation_hash.clone()]);

        self.register_operation(shard, &hash_of_hashes, operations_hashes)
            .await;

        self.remove_fee(hash_of_hashes, operation, shard).await;
        self.common_state().set_fee_status_for_shard(shard, false);
    }

    async fn set_fee_wrapper(&mut self, fee: FeeStruct<StaticApi>, shard: u32) {
        let fee_activated = self.common_state().get_fee_status_for_shard(shard);

        if fee_activated {
            return;
        }

        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let operation: SetFeeOperation<StaticApi> = SetFeeOperation {
            fee_struct: fee.clone(),
            nonce: self
                .common_state()
                .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
        };

        let operation_hash = operation.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        let operations_hashes = MultiValueEncoded::from_iter(vec![operation_hash.clone()]);

        self.register_operation(shard, &hash_of_hashes, operations_hashes)
            .await;

        self.set_fee(hash_of_hashes, operation, shard).await;
        self.common_state().set_fee_status_for_shard(shard, true);
    }

    async fn get_native_token(&mut self, shard: u32) -> EgldOrEsdtTokenIdentifier<StaticApi> {
        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();

        self.interactor()
            .query()
            .to(mvx_esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .native_token()
            .returns(ReturnsResult)
            .run()
            .await
    }
}
