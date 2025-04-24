#![allow(non_snake_case)]

mod config;

use config::Config;
use multiversx_sc_snippets::{imports::*, sdk::bech32};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    chain_factory_proxy::ChainFactoryContractProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    header_verifier_proxy::HeaderverifierProxy,
    mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    sovereign_forge_proxy::SovereignForgeProxy,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
use structs::configs::{EsdtSafeConfig, SovereignConfig};

const STATE_FILE: &str = "state.toml";
const CHAIN_CONFIG_CODE_PATH: &str = "../../chain-config/output/chain-config.mxsc.json";
const CHAIN_FACTORY_CODE_PATH: &str = "../../chain-factory/output/chain-factory.mxsc.json";
const HEADER_VERIFIER_CODE_PATH: &str = "../../header-verifier/output/header-verifier.mxsc.json";
const ESDT_SAFE_CODE_PATH: &str = "../../esdt-safe/output/esdt-safe.mxsc.json";
const FEE_MARKET_CODE_PATH: &str = "../../fee-market/output/fee-market.mxsc.json";

pub async fn sovereign_forge_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "upgrade" => interact.upgrade().await,
        "completeSetupPhase" => interact.complete_setup_phase().await,
        "deployPhaseOne" => interact.deploy_phase_one().await,
        "deployPhaseTwo" => interact.deploy_phase_two().await,
        "deployPhaseThree" => interact.deploy_phase_three().await,
        "getChainFactoryAddress" => interact.chain_factories().await,
        "getTokenHandlerAddress" => interact.token_handlers().await,
        "getDeployCost" => interact.deploy_cost().await,
        "getAllChainIds" => interact.chain_ids().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State {
    contract_address: Option<Bech32Address>,
    config_address: Option<Bech32Address>,
    factory_address: Option<Bech32Address>,
    header_verifier_address: Option<Bech32Address>,
    esdt_safe_address: Option<Bech32Address>,
    fee_market_address: Option<Bech32Address>,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the Sovereign-Forge contract address
    pub fn set_address(&mut self, address: Bech32Address) {
        self.contract_address = Some(address);
    }

    /// Sets the Chain-Config contract address
    pub fn set_config_template(&mut self, address: Bech32Address) {
        self.config_address = Some(address);
    }

    /// Sets the Chain-Factory contract address
    pub fn set_factory_template(&mut self, address: Bech32Address) {
        self.factory_address = Some(address);
    }

    /// Sets the Header-Verifier contract address
    pub fn set_header_verifier_address(&mut self, address: Bech32Address) {
        self.header_verifier_address = Some(address);
    }

    /// Sets the Esdt-Safe contract address
    pub fn set_esdt_safe_address(&mut self, address: Bech32Address) {
        self.esdt_safe_address = Some(address);
    }

    /// Sets the Fee-Market contract address
    pub fn set_fee_market_address(&mut self, address: Bech32Address) {
        self.fee_market_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}

pub struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State,
}

impl ContractInteract {
    pub async fn new() -> Self {
        let config = Config::new();
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("sovereign_forge/interactor");
        let wallet_address = interactor.register_wallet(test_wallets::alice()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/sovereign-forge.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state(),
        }
    }

    pub async fn deploy(&mut self) {
        let deploy_cost = BigUint::<StaticApi>::from(100u128);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(50_000_000u64)
            .typed(SovereignForgeProxy)
            .init(deploy_cost)
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new Forge address: {new_address_bech32}");
    }

    pub async fn deploy_chain_factory(&mut self) {
        let header_verifier_managed_address =
            self.convert_address_to_managed(self.state.header_verifier_address.clone());
        let forge_managed_address =
            self.convert_address_to_managed(self.state.contract_address.clone());
        let config_managed_address =
            self.convert_address_to_managed(self.state.config_address.clone());
        let esdt_safe_managed_address =
            self.convert_address_to_managed(self.state.esdt_safe_address.clone());
        let fee_market_mananged_address =
            self.convert_address_to_managed(self.state.fee_market_address.clone());

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(50_000_000u64)
            .typed(ChainFactoryContractProxy)
            .init(
                forge_managed_address.clone(),
                config_managed_address,
                header_verifier_managed_address,
                esdt_safe_managed_address,
                fee_market_mananged_address,
            )
            .code(MxscPath::new(CHAIN_FACTORY_CODE_PATH))
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_factory_template(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Chain-Factory address: {new_address_bech32}");
    }

    pub fn convert_address_to_managed(
        &mut self,
        address: Option<Bech32Address>,
    ) -> ManagedAddress<StaticApi> {
        let address_bech32 = address.as_ref().unwrap();

        ManagedAddress::from(address_bech32.to_address())
    }

    pub async fn deploy_chain_config_template(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(50_000_000u64)
            .typed(ChainConfigContractProxy)
            .init(SovereignConfig::default_config(), &self.wallet_address)
            .returns(ReturnsNewAddress)
            .code(MxscPath::new(CHAIN_CONFIG_CODE_PATH))
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_config_template(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Chain-Config address: {new_address_bech32}");
    }

    pub async fn deploy_header_verifier_template(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(50_000_000u64)
            .typed(HeaderverifierProxy)
            .init(self.state.config_address.clone().unwrap())
            .returns(ReturnsNewAddress)
            .code(MxscPath::new(HEADER_VERIFIER_CODE_PATH))
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_header_verifier_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Header-Verifier address: {new_address_bech32}");
    }

    pub async fn deploy_esdt_safe_template(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(
                ManagedAddress::from(
                    self.state
                        .header_verifier_address
                        .as_ref()
                        .unwrap()
                        .to_address(),
                ),
                OptionalValue::<EsdtSafeConfig<StaticApi>>::None,
            )
            .returns(ReturnsNewAddress)
            .code(MxscPath::new(ESDT_SAFE_CODE_PATH))
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_esdt_safe_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new ESDT-Safe address: {new_address_bech32}");
    }

    pub async fn deploy_fee_market_template(&mut self) {
        let esdt_safe_managed_address =
            self.convert_address_to_managed(self.state.esdt_safe_address.clone());
        let fee: Option<FeeStruct<StaticApi>> = None;

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(80_000_000u64)
            .typed(FeeMarketProxy)
            .init(esdt_safe_managed_address, fee)
            .returns(ReturnsNewAddress)
            .code(MxscPath::new(FEE_MARKET_CODE_PATH))
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_fee_market_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Fee-Market address: {new_address_bech32}");
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(50_000_000u64)
            .typed(SovereignForgeProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_token_handler(&mut self, shard_id: u32) {
        let bech32 = &self.state.contract_address.as_ref().unwrap();
        let address = bech32.to_address();
        let token_handler_address = ManagedAddress::from(address);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_token_handler(shard_id, token_handler_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_chain_factory(&mut self, shard_id: u32) {
        let bech32 = &self.state.factory_address.as_ref().unwrap();
        let address = bech32.to_address();
        let chain_factory_address = ManagedAddress::from(address);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, chain_factory_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn complete_setup_phase(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deploy_phase_one(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(100u128);

        let config = SovereignConfig::new(0, 1, BigUint::default(), None);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(100_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_one(None::<ManagedBuffer<StaticApi>>, config)
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deploy_phase_two(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_two()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deploy_phase_three(&mut self) {
        let is_sovereign_chain = false;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(80_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_three(is_sovereign_chain)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deploy_phase_four(&mut self) {
        let fee: Option<FeeStruct<StaticApi>> = None;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(80_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_four(fee)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }
    pub async fn chain_factories(&mut self) {
        let shard_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(SovereignForgeProxy)
            .chain_factories(shard_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn token_handlers(&mut self) {
        let shard_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(SovereignForgeProxy)
            .token_handlers(shard_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn deploy_cost(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(SovereignForgeProxy)
            .deploy_cost()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn chain_ids(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(SovereignForgeProxy)
            .chain_ids()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
