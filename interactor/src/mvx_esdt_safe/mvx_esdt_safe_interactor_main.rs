use multiversx_sc_snippets::imports::*;
use operation::EsdtSafeConfig;
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;

use crate::{config::Config, State};

pub struct MvxEsdtSafeInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State,
}

impl MvxEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("mvx-esdt-safe");
        let wallet_address = interactor.register_wallet(test_wallets::alice()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/mvx-esdt-safe.mxsc.json",
            &InterpreterContext::default(),
        );

        MvxEsdtSafeInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state(),
        }
    }

    pub async fn deploy(&mut self) {
        let header_verifier_address = bech32::decode("");
        let opt_config = OptionalValue::Some(EsdtSafeConfig::<StaticApi>::default_config());

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(header_verifier_address, opt_config)
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new address: {new_address_bech32}");
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn update_configuration(&mut self) {
        let new_config = EsdtSafeConfig::<StaticApi>::default_config();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_configuration(new_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee_market_address(&mut self) {
        let fee_market_address = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    // pub async fn deposit(&mut self) {
    //     let token_id = String::new();
    //     let token_nonce = 0u64;
    //     let token_amount = BigUint::<StaticApi>::from(0u128);
    //
    //     let to = bech32::decode("");
    //
    //     let response = self
    //         .interactor
    //         .tx()
    //         .from(&self.wallet_address)
    //         .to(self.state.current_address())
    //         .gas(30_000_000u64)
    //         .typed(MvxEsdtSafeProxy)
    //         .deposit(to, OptionalValue::None)
    //         .payment((
    //             TokenIdentifier::from(token_id.as_str()),
    //             token_nonce,
    //             token_amount,
    //         ))
    //         .returns(ReturnsResultUnmanaged)
    //         .run()
    //         .await;
    //
    //     println!("Result: {response:?}");
    // }

    // pub async fn execute_operations(&mut self) {
    //     let hash_of_hashes = ManagedBuffer::new_from_bytes(&b""[..]);
    //     let operation = Operation::<StaticApi>::new();
    //
    //     let response = self
    //         .interactor
    //         .tx()
    //         .from(&self.wallet_address)
    //         .to(self.state.current_address())
    //         .gas(30_000_000u64)
    //         .typed(MvxEsdtSafeProxy)
    //         .execute_operations(hash_of_hashes, operation)
    //         .returns(ReturnsResultUnmanaged)
    //         .run()
    //         .await;
    //
    //     println!("Result: {response:?}");
    // }

    // pub async fn register_token(&mut self) {
    //     let egld_amount = BigUint::<StaticApi>::from(0u128);
    //
    //     let sov_token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
    //     let token_type = EsdtTokenType::<StaticApi>::default();
    //     let token_display_name = ManagedBuffer::new_from_bytes(&b""[..]);
    //     let token_ticker = ManagedBuffer::new_from_bytes(&b""[..]);
    //     let num_decimals = 0u32;
    //
    //     let response = self
    //         .interactor
    //         .tx()
    //         .from(&self.wallet_address)
    //         .to(self.state.current_address())
    //         .gas(30_000_000u64)
    //         .typed(MvxEsdtSafeProxy)
    //         .register_token(
    //             sov_token_id,
    //             token_type,
    //             token_display_name,
    //             token_ticker,
    //             num_decimals,
    //         )
    //         .egld(egld_amount)
    //         .returns(ReturnsResultUnmanaged)
    //         .run()
    //         .await;
    //
    //     println!("Result: {response:?}");
    // }

    pub async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .pause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn unpause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .unpause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn paused_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(MvxEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn set_max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let max_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(MvxEsdtSafeProxy)
            .max_bridged_amount(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
