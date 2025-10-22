use std::path::Path;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use common_test_setup::constants::{
    FEE_MARKET_SHARD_0, FEE_MARKET_SHARD_1, FEE_MARKET_SHARD_2, GAS_LIMIT, MVX_ESDT_SAFE_SHARD_0,
    MVX_ESDT_SAFE_SHARD_1, MVX_ESDT_SAFE_SHARD_2, PER_GAS, PER_TRANSFER, SHARD_1, TESTING_SC,
    TESTING_SC_ENDPOINT, UNKNOWN_FEE_MARKET, UNKNOWN_MVX_ESDT_SAFE, USER_ADDRESS_STR,
};
use error_messages::{AMOUNT_IS_TOO_LARGE, FAILED_TO_PARSE_AS_NUMBER};
use multiversx_sc::{
    codec::{num_bigint, TopEncode},
    imports::{Bech32Address, MultiValue3, OptionalValue},
    types::{
        Address, BigUint, EgldOrEsdtTokenPayment, EsdtTokenData, EsdtTokenType, ManagedAddress,
        ManagedBuffer, ManagedVec, MultiValueEncoded,
    },
};
use multiversx_sc_snippets::{
    hex,
    imports::{StaticApi, Wallet},
    multiversx_sc_scenario::{
        multiversx_chain_vm::crypto_functions::sha256,
        scenario_model::{Log, TxResponseStatus},
    },
    test_wallets, Interactor,
};
use rand::{distr::Alphanumeric, Rng};
use structs::{
    aliases::PaymentsVec,
    fee::{FeeStruct, FeeType},
    forge::{ContractInfo, ScArray},
    operation::{Operation, OperationData, OperationEsdtPayment, TransferData},
};

use crate::{
    interactor_common_state::CommonState,
    interactor_state::{EsdtTokenInfo, State},
    interactor_structs::{ActionConfig, BalanceCheckConfig, SerializableToken},
};

#[allow(clippy::type_complexity)]
#[allow(async_fn_in_trait)]
pub trait InteractorHelpers {
    fn interactor(&mut self) -> &mut Interactor;
    fn state(&mut self) -> &mut State;
    fn common_state(&mut self) -> &mut CommonState;
    fn user_address(&self) -> &Address;

    fn prepare_transfer_data(
        &self,
        with_transfer_data: bool,
    ) -> OptionalValue<
        MultiValue3<
            u64,
            ManagedBuffer<StaticApi>,
            MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
        >,
    > {
        if with_transfer_data {
            let function = ManagedBuffer::<StaticApi>::from(TESTING_SC_ENDPOINT);
            let args =
                MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
                    vec![ManagedBuffer::from("1")],
                ));
            OptionalValue::Some(MultiValue3::from((GAS_LIMIT, function, args)))
        } else {
            OptionalValue::None
        }
    }

    fn prepare_deposit_payments(
        &mut self,
        token: Option<EsdtTokenInfo>,
        fee: Option<FeeStruct<StaticApi>>,
        with_transfer_data: bool,
    ) -> PaymentsVec<StaticApi> {
        let mut payment_vec = PaymentsVec::new();
        let fee_amount;

        // Add fee payment if present
        if let Some(fee_struct) = fee {
            fee_amount = self.calculate_fee_amount(fee_struct, with_transfer_data, token.clone());

            if fee_amount > 0u64 {
                let fee_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
                    self.state().get_fee_token_identifier(),
                    0,
                    fee_amount.clone(),
                );
                payment_vec.push(fee_payment);
            }
        }

        // Add token payment if present
        if let Some(token) = token {
            let token_payment = EgldOrEsdtTokenPayment::<StaticApi>::new(
                token.token_id,
                token.nonce,
                token.amount.clone(),
            );
            payment_vec.push(token_payment);
        }

        payment_vec
    }

    fn prepare_execute_payment(
        &self,
        token: Option<EsdtTokenInfo>,
    ) -> ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>> {
        match token {
            Some(token) => {
                let token_data = EsdtTokenData {
                    amount: token.amount,
                    token_type: token.token_type,
                    ..Default::default()
                };

                let payment = OperationEsdtPayment::new(token.token_id, token.nonce, token_data);

                let mut payments = ManagedVec::new();
                payments.push(payment);
                payments
            }
            _ => ManagedVec::new(),
        }
    }

    async fn prepare_operation(
        &mut self,
        shard: u32,
        token: Option<EsdtTokenInfo>,
        endpoint: Option<&str>,
    ) -> Operation<StaticApi> {
        let user_address = self.user_address().clone();

        let payment_vec = self.prepare_execute_payment(token);
        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();

        match endpoint {
            Some(endpoint) => {
                let gas_limit = 90_000_000u64;
                let function = ManagedBuffer::<StaticApi>::from(endpoint);
                let args = ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![
                    ManagedBuffer::from("1"),
                ]);

                let transfer_data = TransferData::new(gas_limit, function, args);
                let operation_data = OperationData::new(
                    self.common_state()
                        .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
                    ManagedAddress::from_address(&user_address),
                    Some(transfer_data),
                );

                Operation::new(
                    ManagedAddress::from_address(
                        &self
                            .common_state()
                            .current_testing_sc_address()
                            .to_address(),
                    ),
                    payment_vec,
                    operation_data,
                )
            }
            None => {
                let operation_data = OperationData::new(
                    self.common_state()
                        .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
                    ManagedAddress::from_address(&user_address),
                    None,
                );

                Operation::new(
                    ManagedAddress::from_address(self.user_address()),
                    payment_vec,
                    operation_data,
                )
            }
        }
    }

    fn get_address_name(&mut self, address: &Bech32Address) -> &'static str {
        let testing_addr = self.common_state().current_testing_sc_address();
        if address == testing_addr {
            return TESTING_SC;
        }

        let user_address = self.user_address();
        if address == user_address {
            return USER_ADDRESS_STR;
        }

        // Check shard-specific contract addresses
        for shard_id in 0..3 {
            let mvx_addr = self.common_state().get_mvx_esdt_safe_address(shard_id);
            if address == mvx_addr {
                return match shard_id {
                    0 => MVX_ESDT_SAFE_SHARD_0,
                    1 => MVX_ESDT_SAFE_SHARD_1,
                    2 => MVX_ESDT_SAFE_SHARD_2,
                    _ => UNKNOWN_MVX_ESDT_SAFE,
                };
            }

            let fee_addr = self.common_state().get_fee_market_address(shard_id);
            if address == fee_addr {
                return match shard_id {
                    0 => FEE_MARKET_SHARD_0,
                    1 => FEE_MARKET_SHARD_1,
                    2 => FEE_MARKET_SHARD_2,
                    _ => UNKNOWN_FEE_MARKET,
                };
            }
        }

        "Unknown Address"
    }

    fn calculate_fee_amount(
        &self,
        fee_struct: FeeStruct<StaticApi>,
        with_transfer_data: bool,
        token: Option<EsdtTokenInfo>,
    ) -> BigUint<StaticApi> {
        match &fee_struct.fee_type {
            FeeType::Fixed {
                per_transfer,
                per_gas,
                ..
            } => {
                match (with_transfer_data, token.is_some()) {
                    (true, true) => per_transfer.clone() + per_gas.clone() * GAS_LIMIT, // Transfer + SC call
                    (true, false) => per_gas.clone() * GAS_LIMIT, // SC call only
                    (false, _) => per_transfer.clone(),           // Transfer only
                }
            }
            FeeType::None => BigUint::zero(),
        }
    }

    fn generate_nonce_and_decimals(&mut self, token_type: EsdtTokenType) -> (u64, usize) {
        match token_type {
            EsdtTokenType::Fungible => (0, 18),
            EsdtTokenType::MetaFungible | EsdtTokenType::DynamicMeta => (10, 18),
            EsdtTokenType::NonFungible
            | EsdtTokenType::NonFungibleV2
            | EsdtTokenType::DynamicNFT
            | EsdtTokenType::SemiFungible
            | EsdtTokenType::DynamicSFT => (10, 0),
            _ => panic!("Unsupported token type for getting decimals and nonce"),
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

    fn get_bridge_service_for_shard(&mut self, shard_id: u32) -> Address {
        match shard_id {
            0 => test_wallets::bob().to_address(),
            1 => test_wallets::dan().to_address(),
            2 => test_wallets::heidi().to_address(),
            _ => panic!("Invalid shard ID: {shard_id}"),
        }
    }
    fn get_bridge_owner_for_shard(&mut self, shard_id: u32) -> Address {
        match shard_id {
            0 => test_wallets::mike().to_address(),
            1 => test_wallets::eve().to_address(),
            2 => test_wallets::judy().to_address(),
            _ => panic!("Invalid shard ID: {shard_id}"),
        }
    }

    fn get_sovereign_owner_for_shard(&mut self, shard_id: u32) -> Address {
        match shard_id {
            0 => {
                let wallet_path = "wallets/wallet_shard_0.pem".to_string();
                let wallet = Wallet::from_pem_file(&wallet_path)
                    .unwrap_or_else(|_| panic!("Failed to load wallet for shard {}", shard_id));
                wallet.to_address()
            }
            1 => test_wallets::frank().to_address(),
            2 => test_wallets::carol().to_address(),
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

    fn clone_token_with_amount(
        &mut self,
        token: EsdtTokenInfo,
        new_amount: BigUint<StaticApi>,
    ) -> EsdtTokenInfo {
        EsdtTokenInfo {
            token_id: token.token_id,
            amount: new_amount,
            nonce: token.nonce,
            decimals: token.decimals,
            token_type: token.token_type,
        }
    }

    fn is_sovereign_token(&self, token: &EsdtTokenInfo) -> bool {
        token
            .token_id
            .clone()
            .into_managed_buffer()
            .to_string()
            .matches('-')
            .count()
            == 2
    }

    fn search_for_error_in_logs(&self, logs: &[Log], expected_error_bytes: &[u8]) -> bool {
        logs.iter().any(|log| {
            log.data.iter().any(|data_item| {
                if let Ok(decoded_data) = BASE64.decode(data_item) {
                    decoded_data
                        .windows(expected_error_bytes.len())
                        .any(|w| w == expected_error_bytes)
                } else {
                    false
                }
            })
        })
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
                // If expecting an error, just check it exists. Otherwise, no logs allowed.
                if let Some(expected_error) = expected_log_error {
                    let expected_error_bytes = expected_error.as_bytes();
                    let found_error = self.search_for_error_in_logs(&logs, expected_error_bytes);
                    assert!(found_error, "Expected error '{}' not found", expected_error);
                } else {
                    assert!(logs.is_empty(), "Expected no logs, but found: {:?}", logs);
                }
            }
            Some(expected_log) => {
                if expected_log.is_empty() {
                    return;
                }
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
                    let expected_error_bytes = expected_error.as_bytes();
                    let found_error = self.search_for_error_in_logs(&logs, expected_error_bytes);
                    assert!(found_error, "Expected error '{}' not found", expected_error);
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
            Ok(_) => {
                assert!(
                    expected_error_message.is_none(),
                    "Expected error message: {:?}, but transaction was successful",
                    expected_error_message
                );
            }
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
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
                &self
                    .common_state()
                    .current_chain_config_sc_address()
                    .to_address(),
            ),
            ScArray::ChainFactory => ManagedAddress::from_address(
                &self
                    .common_state()
                    .current_chain_factory_sc_address()
                    .to_address(),
            ),
            ScArray::ESDTSafe => ManagedAddress::from_address(
                &self
                    .common_state()
                    .current_mvx_esdt_safe_contract_address()
                    .to_address(),
            ),
            ScArray::HeaderVerifier => ManagedAddress::from_address(
                &self
                    .common_state()
                    .current_header_verifier_address()
                    .to_address(),
            ),
            ScArray::FeeMarket => ManagedAddress::from_address(
                &self
                    .common_state()
                    .current_fee_market_address()
                    .to_address(),
            ),
        }
    }

    fn get_token_decimals(&self, token_type: EsdtTokenType) -> usize {
        match token_type {
            EsdtTokenType::NonFungibleV2
            | EsdtTokenType::DynamicNFT
            | EsdtTokenType::SemiFungible
            | EsdtTokenType::NonFungible
            | EsdtTokenType::DynamicSFT => 0,
            EsdtTokenType::Fungible | EsdtTokenType::MetaFungible | EsdtTokenType::DynamicMeta => {
                18
            }
            _ => panic!("Unsupported token type for getting decimals"),
        }
    }

    fn get_token_by_type(&mut self, token_type: EsdtTokenType, index: usize) -> EsdtTokenInfo {
        match token_type {
            EsdtTokenType::NonFungibleV2 => self.state().get_nft_token_by_index(index),
            EsdtTokenType::Fungible => self.state().get_fungible_token_by_index(index),
            EsdtTokenType::SemiFungible => self.state().get_sft_token_by_index(index),
            EsdtTokenType::MetaFungible => self.state().get_meta_esdt_token_by_index(index),
            EsdtTokenType::DynamicNFT => self.state().get_dynamic_nft_token_by_index(index),
            EsdtTokenType::DynamicSFT => self.state().get_dynamic_sft_token_by_index(index),
            EsdtTokenType::DynamicMeta => self.state().get_dynamic_meta_esdt_token_by_index(index),
            _ => panic!("Unsupported token type for test"),
        }
    }

    fn extract_log_based_on_shard(&mut self, config: &ActionConfig) -> Option<String> {
        match &config.expected_log {
            Some(logs) if logs.len() == 1 => Some(logs[0].clone()),
            Some(logs) if logs.len() > 1 => match config.shard {
                SHARD_1 => Some(logs[0].clone()),
                _ => Some(logs[1].clone()),
            },
            _ => None,
        }
    }

    // CHECK BALANCE OPERATIONS

    async fn check_address_balance(
        &mut self,
        address: &Bech32Address,
        expected_token_balance: Vec<EsdtTokenInfo>,
    ) {
        let address_name = self.get_address_name(address);

        let balances = self
            .interactor()
            .get_account_esdt(&address.to_address())
            .await;

        if expected_token_balance.is_empty() {
            assert!(
                balances.is_empty(),
                "Expected no tokens for {} ({}), but found: {:?}",
                address_name,
                address,
                balances
            );
            return;
        }

        for token_balance in expected_token_balance {
            let token_id = &token_balance.token_id.into_managed_buffer().to_string();
            let expected_amount = &token_balance.amount;

            if *expected_amount == 0u64 {
                match balances.get(token_id) {
                    None => {}
                    Some(esdt_balance) => {
                        panic!("For {} ({}) -> Expected token '{}' to be absent (balance 0), but found it with balance: {}",
                           address_name, address, token_id, esdt_balance.balance);
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
                    "\nFor {} ({}) -> Balance mismatch for token {}:\nexpected: {}\nfound:    {}",
                    address_name,
                    address,
                    found_token_id,
                    expected_amount.to_display(),
                    esdt_balance.balance
                );
                }
                None => {
                    panic!(
                    "For {} ({}) -> Expected token starting with '{}' with balance {}, but none was found",
                    address_name,
                    address,
                    token_id,
                    expected_amount.to_display()
                );
                }
            }
        }
    }

    async fn check_user_balance(&mut self, expected_tokens: Vec<EsdtTokenInfo>) {
        let user_address = Bech32Address::from(self.user_address().clone());
        self.check_address_balance(&user_address, expected_tokens)
            .await;
    }

    async fn check_mvx_esdt_balance(&mut self, shard: u32, expected_tokens: Vec<EsdtTokenInfo>) {
        let mvx_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let tokens = if expected_tokens.is_empty() {
            self.create_empty_balance_state().await
        } else {
            expected_tokens
        };
        self.check_address_balance(&mvx_address, tokens).await;
    }

    async fn check_address_egld_balance(&mut self, address: &Bech32Address, expected_amount: u64) {
        let balance = self
            .interactor()
            .get_account(&address.clone().into_address())
            .await
            .balance;
        assert_eq!(
            balance,
            expected_amount.to_string(),
            "EGLD balance mismatch for {:?} :\n",
            address,
        );
    }

    async fn check_fee_market_balance(&mut self, shard: u32, expected_tokens: Vec<EsdtTokenInfo>) {
        let fee_market_address = self.common_state().get_fee_market_address(shard).clone();
        let tokens = if expected_tokens.is_empty() {
            vec![self
                .common_state()
                .get_fee_market_token_for_shard_converted(shard)]
        } else {
            expected_tokens
        };
        self.check_address_balance(&fee_market_address, tokens)
            .await;
    }

    async fn check_testing_sc_balance(&mut self, expected_tokens: Vec<EsdtTokenInfo>) {
        let testing_sc_address = self.common_state().current_testing_sc_address().clone();
        let tokens = if expected_tokens.is_empty() {
            self.create_empty_balance_state().await
        } else {
            expected_tokens
        };
        self.check_address_balance(&testing_sc_address, tokens)
            .await;
    }

    async fn check_user_balance_unchanged(&mut self) {
        let expected_balance = self.state().get_initial_wallet_tokens_state().clone();
        self.check_user_balance(expected_balance).await;
    }

    async fn check_contracts_empty(&mut self, shard: u32) {
        self.check_mvx_esdt_balance(shard, Vec::new()).await;
        self.check_testing_sc_balance(Vec::new()).await;
    }

    async fn create_empty_balance_state(&mut self) -> Vec<EsdtTokenInfo> {
        let mut empty_balance_state = self.state().get_initial_wallet_tokens_state().clone();
        for token in empty_balance_state.iter_mut() {
            token.amount = BigUint::from(0u64);
        }
        empty_balance_state
    }

    fn create_serializable_token(
        &mut self,
        token: EsdtTokenInfo,
        amount: u64,
    ) -> SerializableToken {
        SerializableToken {
            token_id: token.token_id.into_managed_buffer().to_string(),
            token_type: token.token_type as u8,
            nonce: token.nonce,
            decimals: token.decimals,
            amount,
        }
    }

    /// For user we have two cases:
    /// 1. User should get tokens back after execute call (with_transfer_data = false)
    /// 2. User should not get tokens back after execute call (with_transfer_data = true)
    ///
    /// For MVX we have two cases:
    /// 1. Tokens are deposited to MVX ESDT Safe
    /// 2. Tokens leave the contract after operations or are burned (special case for the 1 SFT/META sov token that stays in the sc)
    async fn check_balances_after_action(&mut self, bcc: BalanceCheckConfig) {
        let BalanceCheckConfig {
            shard,
            token,
            amount,
            fee,
            with_transfer_data,
            is_execute,
            expected_error,
        } = bcc;

        let is_sov_mapped_token = token
            .as_ref()
            .map(|t| {
                t.clone()
                    .token_id
                    .into_managed_buffer()
                    .to_string()
                    .split('-')
                    .next()
                    == Some("SOV")
            })
            .unwrap_or(false);

        let is_egld = token.clone().map(|t| t.token_id.is_egld()).unwrap_or(false);

        let fee_amount = fee
            .as_ref()
            .map(|f| self.calculate_fee_amount(f.clone(), with_transfer_data, token.clone()))
            .unwrap_or_else(BigUint::zero);

        let mut expected_user_tokens = Vec::new();

        // USER tokens
        if let (Some(token), Some(amount)) = (token.clone(), amount.clone()) {
            let token_id = token.token_id.clone();
            let initial_user_balance = self
                .state()
                .get_initial_wallet_token_balance(token_id.clone());

            let user_should_get_token_back = is_execute && !with_transfer_data;

            let remaining_amount = match (user_should_get_token_back, is_sov_mapped_token) {
                (true, true) => amount,
                (true, false) => initial_user_balance,
                (false, _) => Self::safe_subtract(initial_user_balance, amount.clone()),
            };

            expected_user_tokens
                .push(self.clone_token_with_amount(token.clone(), remaining_amount));
        }

        if fee.is_some() && fee_amount > 0u64 {
            let fee_token = self.state().get_fee_token_id();
            let initial_fee_balance = fee_token.clone().amount;
            let remaining_fee = Self::safe_subtract(initial_fee_balance, fee_amount.clone());
            expected_user_tokens.push(self.clone_token_with_amount(fee_token, remaining_fee));
        }

        if expected_user_tokens.is_empty() || expected_error.is_some() {
            self.check_user_balance_unchanged().await;
        } else {
            self.check_user_balance(expected_user_tokens).await;
        }

        //MVX Tokens
        if is_egld {
            let current_balance = self.common_state().get_mvx_egld_balance_for_shard(shard);
            let amount_u64 = amount.clone().unwrap().to_u64().expect(AMOUNT_IS_TOO_LARGE);
            let expected_amount = if is_execute {
                current_balance - amount_u64
            } else {
                current_balance + amount_u64
            };
            let address = self.common_state().get_mvx_esdt_safe_address(shard).clone();
            self.check_address_egld_balance(&address, expected_amount)
                .await;
            self.common_state()
                .update_mvx_egld_balance_with_amount(shard, expected_amount);
        } else {
            // ESDT tokens
            let mvx_tokens = match (&token, &amount, is_sov_mapped_token, is_execute) {
                (Some(token), Some(_), true, _) => {
                    // Sovereign mapped tokens: only keep 1 SFT/META token in the contract
                    if matches!(
                        token.token_type,
                        EsdtTokenType::MetaFungible
                            | EsdtTokenType::DynamicMeta
                            | EsdtTokenType::DynamicSFT
                            | EsdtTokenType::SemiFungible
                    ) {
                        vec![self.clone_token_with_amount(token.clone(), BigUint::from(1u64))]
                    } else {
                        vec![]
                    }
                }
                (Some(token), Some(amount), false, false) => {
                    // Non-sovereign deposits: full amount goes to MVX safe
                    vec![self.clone_token_with_amount(token.clone(), amount.clone())]
                }
                _ => vec![],
            };
            self.check_mvx_esdt_balance(shard, mvx_tokens).await;
        }

        // FEE market
        if fee_amount > 0u64 {
            let fee_token = self.state().get_fee_token_id();
            let previous_fee_amount = BigUint::from(
                self.common_state()
                    .get_fee_market_token_amount_for_shard(shard),
            );
            let expected_fee_tokens =
                vec![self.clone_token_with_amount(fee_token, previous_fee_amount + fee_amount)];
            self.check_fee_market_balance(shard, expected_fee_tokens)
                .await;
        } else {
            self.check_fee_market_balance(shard, vec![]).await;
        }

        // TESTING SC
        if is_egld && is_execute && with_transfer_data && expected_error.is_none() {
            let expected_amount = self.common_state().get_testing_egld_balance()
                + amount.clone().unwrap().to_u64().unwrap();
            let testing_address = self.common_state().current_testing_sc_address().clone();
            self.check_address_egld_balance(&testing_address, expected_amount)
                .await;
            self.common_state()
                .update_testing_egld_balance_with_amount(expected_amount);
        } else {
            let testing_sc_tokens = match (&token, &amount) {
                (Some(token), Some(amount)) => {
                    if is_execute && with_transfer_data && expected_error.is_none() {
                        vec![self.clone_token_with_amount(token.clone(), amount.clone())]
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            };

            self.check_testing_sc_balance(testing_sc_tokens).await;
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

    fn safe_subtract(a: BigUint<StaticApi>, b: BigUint<StaticApi>) -> BigUint<StaticApi> {
        if a > b {
            a - b
        } else {
            BigUint::zero()
        }
    }

    fn is_nft(&self, token: &EsdtTokenInfo) -> bool {
        matches!(
            token.token_type,
            EsdtTokenType::NonFungibleV2 | EsdtTokenType::DynamicNFT | EsdtTokenType::NonFungible
        )
    }

    fn generate_random_chain_id() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| c.is_ascii_alphabetic() && c.is_ascii_lowercase())
            .take(4)
            .map(char::from)
            .collect()
    }

    fn load_wallet(wallet_path: &Path, test_id: u64) -> Wallet {
        if wallet_path.exists() {
            Wallet::from_pem_file(wallet_path.to_str().unwrap()).unwrap_or_else(|_| {
                panic!(
                    "Failed to load {} for test {}",
                    wallet_path.display(),
                    test_id
                )
            })
        } else {
            panic!("{} not found for test {}", wallet_path.display(), test_id);
        }
    }

    fn create_random_sovereign_token_id(&mut self, shard: u32) -> String {
        let current_chain_id = self.common_state().get_chain_id_for_shard(shard).clone();
        let rand_string: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| c.is_ascii_alphanumeric() && c.is_ascii_lowercase())
            .take(6)
            .map(char::from)
            .collect();
        format!("{}-SOV-{}", current_chain_id, rand_string)
    }

    async fn update_fee_market_balance_state(
        &mut self,
        fee: Option<FeeStruct<StaticApi>>,
        payment_vec: PaymentsVec<StaticApi>,
        shard: u32,
    ) {
        if fee.is_none() || payment_vec.is_empty() {
            return;
        }
        let mut fee_token_in_fee_market = self.common_state().get_fee_market_token_for_shard(shard);

        let payment = payment_vec.get(0);
        if let Some(payment_amount) = payment.amount.to_u64() {
            fee_token_in_fee_market.amount += payment_amount;
        }
        self.common_state()
            .set_fee_market_token_for_shard(shard, fee_token_in_fee_market);
    }
}
