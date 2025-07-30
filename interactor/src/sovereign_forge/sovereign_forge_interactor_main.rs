#![allow(non_snake_case)]
use common_interactor::common_sovereign_interactor::{IssueTokenStruct, MintTokenStruct};
use common_interactor::interactor_state::{EsdtTokenInfo, State};
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::{
    DEPLOY_COST, GAS_LIMIT, INTERACTOR_WORKING_DIR, ONE_THOUSAND_TOKENS,
    OPERATION_HASH_STATUS_STORAGE_KEY, SOVEREIGN_FORGE_CODE_PATH, SOVEREIGN_RECEIVER_ADDRESS,
    TESTING_SC_ENDPOINT,
};
use header_verifier::OperationHashStatus;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_snippets::{hex, imports::*};
use proxies::fee_market_proxy::FeeMarketProxy;
use proxies::sovereign_forge_proxy::SovereignForgeProxy;
use structs::aliases::PaymentsVec;
use structs::fee::{FeeStruct, FeeType};
use structs::operation::{Operation, OperationData, OperationEsdtPayment, TransferData};

pub struct SovereignForgeInteract {
    pub interactor: Interactor,
    pub user_address: Address,
    pub state: State,
}
impl CommonInteractorTrait for SovereignForgeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }

    fn user_address(&self) -> &Address {
        &self.user_address
    }
}
impl SovereignForgeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config.clone()).await;

        interactor.register_wallets().await;

        match config.use_chain_simulator() {
            true => {
                interactor.initialize_tokens_in_wallets().await;
            }
            false => {
                println!("Skipping token initialization for real network");
            }
        }
        interactor
    }

    async fn initialize_interactor(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        let current_working_dir = INTERACTOR_WORKING_DIR;
        interactor.set_current_dir_from_workspace(current_working_dir);

        let user_address = interactor.register_wallet(test_wallets::grace()).await; //shard 1

        interactor.generate_blocks_until_epoch(1).await.unwrap();

        SovereignForgeInteract {
            interactor,
            user_address,
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
            token_type: EsdtTokenType::NonFungibleV2,
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
            token_type: EsdtTokenType::MetaFungible,
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

        let dyn_sft_token_struct = IssueTokenStruct {
            token_display_name: "DYNS".to_string(),
            token_ticker: "DYNS".to_string(),
            token_type: EsdtTokenType::DynamicSFT,
            num_decimals: 18,
        };
        let dyn_sft_token_mint = MintTokenStruct {
            name: Some("DYNS".to_string()),
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };

        let dyn_sft_token = self
            .issue_and_mint_token(dyn_sft_token_struct, dyn_sft_token_mint)
            .await;

        self.state.set_dynamic_sft_token_id(dyn_sft_token);

        let dyn_meta_esdt_token_struct = IssueTokenStruct {
            token_display_name: "DYNM".to_string(),
            token_ticker: "DYNM".to_string(),
            token_type: EsdtTokenType::DynamicMeta,
            num_decimals: 18,
        };

        let dyn_meta_esdt_token_mint = MintTokenStruct {
            name: Some("DYNM".to_string()),
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };

        let dyn_meta_esdt_token = self
            .issue_and_mint_token(dyn_meta_esdt_token_struct, dyn_meta_esdt_token_mint)
            .await;

        self.state
            .set_dynamic_meta_esdt_token_id(dyn_meta_esdt_token);

        let initial_tokens_wallet = vec![
            self.state.get_first_token_id(),
            self.state.get_second_token_id(),
            self.state.get_fee_token_id(),
            self.state.get_nft_token_id(),
            self.state.get_meta_esdt_token_id(),
            self.state.get_dynamic_nft_token_id(),
            self.state.get_sft_token_id(),
            self.state.get_dynamic_meta_esdt_token_id(),
            self.state.get_dynamic_sft_token_id(),
        ];
        self.state.set_initial_wallet_balance(initial_tokens_wallet);
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

    pub async fn get_token_fee(&mut self, shard: u32, token: TokenIdentifier<StaticApi>) {
        let user_address = self.user_address().clone();
        let fee_market_addrress = self.state.get_fee_market_address(shard);
        let response = self
            .interactor
            .tx()
            .from(user_address)
            .to(fee_market_addrress)
            .gas(90_000_000u64)
            .typed(FeeMarketProxy)
            .token_fee(token)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;
        println!("Result: {response:?}");
    }

    pub async fn complete_deposit_flow_with_transfer_data_only(
        &mut self,
        shard: u32,
        fee: Option<FeeStruct<StaticApi>>,
        expected_log: Option<&str>,
    ) {
        let gas_limit = 90_000_000u64;
        let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
        let args = MultiValueEncoded::from(
            ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]),
        );

        let transfer_data = MultiValue3::from((gas_limit, function, args));

        self.deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            fee.clone(),
        )
        .await;

        match fee.as_ref() {
            Some(fee_struct) => {
                let fee_amount = match &fee_struct.fee_type {
                    FeeType::Fixed {
                        per_transfer,
                        per_gas,
                        token: _,
                    }
                    | FeeType::AnyToken {
                        per_transfer,
                        per_gas,
                        base_fee_token: _,
                    } => per_transfer.clone() + per_gas.clone() * gas_limit,
                    FeeType::None => BigUint::zero(),
                };

                let mut payment_vec = PaymentsVec::new();

                let fee_payment = EsdtTokenPayment::<StaticApi>::new(
                    self.state.get_fee_token_identifier(),
                    0,
                    fee_amount.clone(),
                );

                payment_vec.push(fee_payment);
                self.deposit_in_mvx_esdt_safe(
                    SOVEREIGN_RECEIVER_ADDRESS.to_address(),
                    shard,
                    OptionalValue::Some(transfer_data),
                    payment_vec,
                    None,
                    expected_log,
                )
                .await;
            }
            None => {
                self.deposit_in_mvx_esdt_safe(
                    SOVEREIGN_RECEIVER_ADDRESS.to_address(),
                    shard,
                    OptionalValue::Some(transfer_data),
                    PaymentsVec::new(),
                    None,
                    expected_log,
                )
                .await;
            }
        }
    }

    pub async fn deposit_no_transfer_data(
        &mut self,
        shard: u32,
        token: EsdtTokenInfo,
        amount: BigUint<StaticApi>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        match fee.as_ref() {
            Some(fee_struct) => {
                let fee_amount = match &fee_struct.fee_type {
                    FeeType::Fixed { per_transfer, .. } => per_transfer.clone(),
                    FeeType::AnyToken { per_transfer, .. } => per_transfer.clone(),
                    FeeType::None => BigUint::zero(),
                };

                let mut payment_vec = PaymentsVec::new();

                let fee_payment = EsdtTokenPayment::<StaticApi>::new(
                    self.state.get_fee_token_identifier(),
                    0,
                    fee_amount.clone(),
                );

                let token_payment = EsdtTokenPayment::<StaticApi>::new(
                    TokenIdentifier::from_esdt_bytes(&token.token_id),
                    token.nonce,
                    amount.clone(),
                );

                payment_vec.push(fee_payment);
                payment_vec.push(token_payment);

                self.deposit_in_mvx_esdt_safe(
                    SOVEREIGN_RECEIVER_ADDRESS.to_address(),
                    shard,
                    OptionalValue::None,
                    payment_vec,
                    None,
                    Some(&token.token_id),
                )
                .await;

                let fee_token = self.state.get_fee_token_id();

                self.check_fee_market_balance_with_amount(shard, fee_token, fee_amount)
                    .await;
            }
            None => {
                let mut payment_vec = PaymentsVec::new();

                let token_payment = EsdtTokenPayment::<StaticApi>::new(
                    TokenIdentifier::from_esdt_bytes(&token.token_id),
                    token.nonce,
                    amount.clone(),
                );

                payment_vec.push(token_payment);

                self.deposit_in_mvx_esdt_safe(
                    SOVEREIGN_RECEIVER_ADDRESS.to_address(),
                    shard,
                    OptionalValue::None,
                    payment_vec,
                    None,
                    Some(&token.token_id),
                )
                .await;

                self.check_fee_market_balance_is_empty(shard).await;
            }
        }
    }

    pub async fn deposit_with_transfer_data(
        &mut self,
        shard: u32,
        token: EsdtTokenInfo,
        amount: BigUint<StaticApi>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
        let args = MultiValueEncoded::from(
            ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]),
        );

        let transfer_data = MultiValue3::from((GAS_LIMIT, function, args));

        match fee.as_ref() {
            Some(fee_struct) => {
                let fee_amount = match &fee_struct.fee_type {
                    FeeType::Fixed {
                        per_transfer,
                        per_gas,
                        token: _,
                    } => per_transfer.clone() + per_gas.clone() * GAS_LIMIT,
                    FeeType::AnyToken {
                        per_transfer,
                        per_gas,
                        base_fee_token: _,
                    } => per_transfer.clone() + per_gas.clone() * GAS_LIMIT,
                    FeeType::None => BigUint::zero(),
                };

                let mut payment_vec = PaymentsVec::new();

                let fee_payment = EsdtTokenPayment::<StaticApi>::new(
                    self.state.get_fee_token_identifier(),
                    0,
                    fee_amount.clone(),
                );

                let token_payment = EsdtTokenPayment::<StaticApi>::new(
                    TokenIdentifier::from_esdt_bytes(&token.token_id),
                    token.nonce,
                    amount.clone(),
                );

                payment_vec.push(fee_payment);
                payment_vec.push(token_payment);

                self.deposit_in_mvx_esdt_safe(
                    SOVEREIGN_RECEIVER_ADDRESS.to_address(),
                    shard,
                    OptionalValue::Some(transfer_data),
                    payment_vec,
                    None,
                    Some(&token.token_id),
                )
                .await;

                self.check_fee_market_balance_with_amount(
                    shard,
                    self.state.get_fee_token_id(),
                    fee_amount,
                )
                .await;
            }
            None => {
                let mut payment_vec = PaymentsVec::new();

                let token_payment = EsdtTokenPayment::<StaticApi>::new(
                    TokenIdentifier::from_esdt_bytes(&token.token_id),
                    token.nonce,
                    amount.clone(),
                );

                payment_vec.push(token_payment);

                self.deposit_in_mvx_esdt_safe(
                    SOVEREIGN_RECEIVER_ADDRESS.to_address(),
                    shard,
                    OptionalValue::Some(transfer_data),
                    payment_vec,
                    None,
                    Some(&token.token_id),
                )
                .await;

                self.check_fee_market_balance_is_empty(shard).await;
            }
        }
    }

    pub async fn complete_execute_operation_flow_with_transfer_data_only(
        &mut self,
        shard: u32,
        expected_error: Option<&str>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
        endpoint: &str,
    ) {
        let user_address = self.user_address().clone();

        let gas_limit = 90_000_000u64;
        let function = ManagedBuffer::<StaticApi>::from(endpoint);
        let args =
            ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

        let transfer_data = TransferData::new(gas_limit, function, args);

        let operation_data = OperationData::new(
            1,
            ManagedAddress::from_address(&user_address),
            Some(transfer_data),
        );

        self.deploy_and_complete_setup_phase(
            DEPLOY_COST.into(),
            OptionalValue::None,
            OptionalValue::None,
            None,
        )
        .await;

        let operation = Operation::new(
            ManagedAddress::from_address(&self.state.current_testing_sc_address().to_address()),
            ManagedVec::new(),
            operation_data,
        );

        let operation_hash = self.get_operation_hash(&operation);
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

        self.register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

        let operation_status = OperationHashStatus::NotLocked as u8;
        let expected_operation_hash_status = format!("{:02x}", operation_status);
        let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

        self.check_account_storage(
            self.state.get_header_verifier_address(shard).to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

        let caller = self.get_bridge_service_for_shard(shard);
        self.execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            expected_error,
            expected_log,
            expected_log_error,
        )
        .await;

        self.check_account_storage(
            self.state.get_header_verifier_address(shard).to_address(),
            encoded_key,
            None,
        )
        .await;
    }

    pub async fn execute_operation(
        &mut self,
        shard: u32,
        expected_error: Option<&str>,
        expected_log: Option<&str>,
        token: EsdtTokenInfo,
        amount: BigUint<StaticApi>,
        endpoint: Option<&str>,
    ) {
        let user_address = self.user_address().clone();

        let token_data = EsdtTokenData {
            amount,
            token_type: token.token_type,
            ..Default::default()
        };

        let payment = OperationEsdtPayment::new(
            TokenIdentifier::from_esdt_bytes(&token.token_id),
            token.nonce,
            token_data,
        );

        let mut payment_vec = ManagedVec::new();
        payment_vec.push(payment);

        let operation = match endpoint {
            Some(endpoint) => {
                let gas_limit = 90_000_000u64;
                let function = ManagedBuffer::<StaticApi>::from(endpoint);
                let args = ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![
                    ManagedBuffer::from("1"),
                ]);

                let transfer_data = TransferData::new(gas_limit, function, args);

                let operation_data = OperationData::new(
                    1,
                    ManagedAddress::from_address(&user_address),
                    Some(transfer_data),
                );
                Operation::new(
                    ManagedAddress::from_address(
                        &self.state.current_testing_sc_address().to_address(),
                    ),
                    payment_vec,
                    operation_data,
                )
            }
            None => {
                let operation_data =
                    OperationData::new(1, ManagedAddress::from_address(&user_address), None);
                Operation::new(
                    ManagedAddress::from_address(self.user_address()),
                    payment_vec,
                    operation_data,
                )
            }
        };

        let operation_hash = self.get_operation_hash(&operation);
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

        self.register_operation(
            shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

        let operation_status = OperationHashStatus::NotLocked as u8;
        let expected_operation_hash_status = format!("{:02x}", operation_status);
        let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

        self.check_account_storage(
            self.state.get_header_verifier_address(shard).to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

        let caller = self.get_bridge_service_for_shard(shard);
        self.execute_operations_in_mvx_esdt_safe(
            caller,
            shard,
            hash_of_hashes,
            operation,
            expected_error,
            expected_log,
            None,
        )
        .await;

        self.check_account_storage(
            self.state.get_header_verifier_address(shard).to_address(),
            encoded_key,
            None,
        )
        .await;
    }
}
