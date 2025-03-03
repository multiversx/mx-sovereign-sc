use multiversx_sc_snippets::imports::*;
use operation::EsdtSafeConfig;
use proxies::sov_esdt_safe_proxy::SovEsdtSafeProxy;

use crate::{config::Config, State};

pub struct SovEsdtSafeInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State,
}

impl SovEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("sov-esdt-safe");
        let wallet_address = interactor.register_wallet(test_wallets::alice()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/sov-esdt-safe.mxsc.json",
            &InterpreterContext::default(),
        );

        SovEsdtSafeInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state(),
        }
    }

    pub async fn deploy(&mut self) {
        let fee_market_address = bech32::decode("");
        let opt_config = OptionalValue::Some(EsdtSafeConfig::<StaticApi>::default_config());

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(SovEsdtSafeProxy)
            .init(fee_market_address, opt_config)
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
            .typed(SovEsdtSafeProxy)
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
            .typed(SovEsdtSafeProxy)
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
            .typed(SovEsdtSafeProxy)
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
    //     let opt_transfer_data = MultiValue3::<u64, ManagedBuffer<StaticApi>, ManagedVec<StaticApi, ManagedBuffer<StaticApi>>>::from((0u64, ManagedBuffer::new_from_bytes(&b""[..]), ManagedVec::from_single_item(ManagedBuffer::new_from_bytes(&b""[..])))));
    //
    //     let response = self
    //         .interactor
    //         .tx()
    //         .from(&self.wallet_address)
    //         .to(self.state.current_address())
    //         .gas(30_000_000u64)
    //         .typed(SovEsdtSafeProxy)
    //         .deposit(to, opt_transfer_data)
    //         .payment((TokenIdentifier::from(token_id.as_str()), token_nonce, token_amount))
    //         .returns(ReturnsResultUnmanaged)
    //         .run()
    //         .await;
    //
    //     println!("Result: {response:?}");
    // }

    pub async fn set_max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let max_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(SovEsdtSafeProxy)
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
            .typed(SovEsdtSafeProxy)
            .max_bridged_amount(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(SovEsdtSafeProxy)
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
            .typed(SovEsdtSafeProxy)
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
            .typed(SovEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
