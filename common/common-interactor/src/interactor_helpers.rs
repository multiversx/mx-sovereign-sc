use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use common_test_setup::constants::{
    FEE_MARKET_SHARD_0, FEE_MARKET_SHARD_1, FEE_MARKET_SHARD_2, GAS_LIMIT, MVX_ESDT_SAFE_SHARD_0,
    MVX_ESDT_SAFE_SHARD_1, MVX_ESDT_SAFE_SHARD_2, PER_GAS, PER_TRANSFER, SHARD_1, TESTING_SC,
    TESTING_SC_ENDPOINT, UNKNOWN_FEE_MARKET, UNKNOWN_MVX_ESDT_SAFE, USER_ADDRESS_STR, WALLET_PATH,
};
use error_messages::{FAILED_TO_LOAD_WALLET_SHARD_0, FAILED_TO_PARSE_AS_NUMBER};
use multiversx_sc::{
    codec::{num_bigint, TopEncode},
    imports::{Bech32Address, MultiValue3, OptionalValue},
    types::{
        Address, BigUint, EsdtTokenData, EsdtTokenPayment, EsdtTokenType, ManagedAddress,
        ManagedBuffer, ManagedVec, MultiValueEncoded, TestSCAddress, TokenIdentifier,
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
    interactor_state::{EsdtTokenInfo, State},
    interactor_structs::{ActionConfig, BalanceCheckConfig},
};

#[allow(clippy::type_complexity)]
#[allow(async_fn_in_trait)]
pub trait InteractorHelpers {
    fn interactor(&mut self) -> &mut Interactor;
    fn state(&mut self) -> &mut State;
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
                let fee_payment = EsdtTokenPayment::<StaticApi>::new(
                    self.state().get_fee_token_identifier(),
                    0,
                    fee_amount.clone(),
                );
                payment_vec.push(fee_payment);
            }
        }

        // Add token payment if present
        if let Some(token) = token {
            let token_payment = EsdtTokenPayment::<StaticApi>::new(
                TokenIdentifier::from_esdt_bytes(&token.token_id),
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

                let payment = OperationEsdtPayment::new(
                    TokenIdentifier::from_esdt_bytes(&token.token_id),
                    token.nonce,
                    token_data,
                );

                let mut payments = ManagedVec::new();
                payments.push(payment);
                payments
            }
            _ => ManagedVec::new(),
        }
    }

    async fn prepare_operation(
        &mut self,
        token: Option<EsdtTokenInfo>,
        endpoint: Option<&str>,
    ) -> Operation<StaticApi> {
        let user_address = self.user_address().clone();

        let payment_vec = self.prepare_execute_payment(token);

        match endpoint {
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
                        &self.state().current_testing_sc_address().to_address(),
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
        }
    }

    fn get_address_name(&mut self, address: &Bech32Address) -> &'static str {
        let testing_addr = self.state().current_testing_sc_address();
        if address == testing_addr {
            return TESTING_SC;
        }

        let user_address = self.user_address();
        if address == user_address {
            return USER_ADDRESS_STR;
        }

        // Check shard-specific contract addresses
        for shard_id in 0..3 {
            let mvx_addr = self.state().get_mvx_esdt_safe_address(shard_id);
            if address == mvx_addr {
                return match shard_id {
                    0 => MVX_ESDT_SAFE_SHARD_0,
                    1 => MVX_ESDT_SAFE_SHARD_1,
                    2 => MVX_ESDT_SAFE_SHARD_2,
                    _ => UNKNOWN_MVX_ESDT_SAFE,
                };
            }

            let fee_addr = self.state().get_fee_market_address(shard_id);
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

    fn get_bridge_service_for_shard(&self, shard_id: u32) -> Address {
        let shard_0_wallet =
            Wallet::from_pem_file(WALLET_PATH).expect(FAILED_TO_LOAD_WALLET_SHARD_0);
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
        token.token_id.matches('-').count() == 2
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
                            decoded_data
                                .windows(expected_error_bytes.len())
                                .any(|w| w == expected_error_bytes)
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
            _ => TestSCAddress::new("ERROR").to_managed_address(),
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
            let token_id = &token_balance.token_id;
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
        let mvx_address = self.state().get_mvx_esdt_safe_address(shard).clone();
        self.check_address_balance(&mvx_address, expected_tokens)
            .await;
    }

    async fn check_fee_market_balance(&mut self, shard: u32, expected_tokens: Vec<EsdtTokenInfo>) {
        let fee_market_address = self.state().get_fee_market_address(shard).clone();
        self.check_address_balance(&fee_market_address, expected_tokens)
            .await;
    }

    async fn check_testing_sc_balance(&mut self, expected_tokens: Vec<EsdtTokenInfo>) {
        let testing_sc_address = self.state().current_testing_sc_address().clone();
        self.check_address_balance(&testing_sc_address, expected_tokens)
            .await;
    }

    async fn check_user_balance_unchanged(&mut self) {
        let expected_balance = self.state().get_initial_wallet_balance().clone().unwrap();
        self.check_user_balance(expected_balance).await;
    }

    async fn check_all_contracts_empty(&mut self, shard: u32) {
        self.check_mvx_esdt_balance(shard, Vec::new()).await;
        self.check_fee_market_balance(shard, Vec::new()).await;
        self.check_testing_sc_balance(Vec::new()).await;
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

        let is_sov_mapped_token = if token.is_some() {
            token.clone().unwrap().token_id.split('-').nth(0) == Some("SOV")
        } else {
            false
        };

        let fee_amount = fee
            .as_ref()
            .map(|f| self.calculate_fee_amount(f.clone(), with_transfer_data, token.clone()))
            .unwrap_or_else(BigUint::zero);

        let mut expected_user_tokens = Vec::new();

        // USER tokens
        if let (Some(token), Some(amount)) = (token.clone(), amount.clone()) {
            let token_id = TokenIdentifier::from_esdt_bytes(token.token_id.clone());
            let initial_user_balance = self
                .state()
                .get_initial_token_balance_for_wallet(token_id.clone());

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

        // MVX tokens
        let mvx_tokens = match (&token, &amount, is_sov_mapped_token, is_execute) {
            (Some(token), Some(_), true, _) => {
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
            _ => {
                vec![]
            }
        };

        self.check_mvx_esdt_balance(shard, mvx_tokens).await;

        // FEE market
        if fee_amount > 0u64 {
            let fee_token = self.state().get_fee_token_id();
            let expected_fee_tokens = vec![self.clone_token_with_amount(fee_token, fee_amount)];
            self.check_fee_market_balance(shard, expected_fee_tokens)
                .await;
        } else {
            self.check_fee_market_balance(shard, vec![]).await;
        }

        // TESTING SC
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
}
