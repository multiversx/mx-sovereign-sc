#![allow(non_snake_case)]
use common_interactor::common_sovereign_interactor::{IssueTokenStruct, MintTokenStruct};
use common_interactor::interactor_state::State;
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::{
    INTERACTOR_WORKING_DIR, ONE_THOUSAND_TOKENS, SOVEREIGN_FORGE_CODE_PATH,
};
use error_messages::FAILED_TO_LOAD_WALLET_SHARD_0;
use multiversx_sc_snippets::imports::*;
use proxies::sovereign_forge_proxy::SovereignForgeProxy;

pub struct SovereignForgeInteract {
    pub interactor: Interactor,
    pub bridge_owner_shard_0: Address,
    pub bridge_owner_shard_1: Address,
    pub bridge_owner_shard_2: Address,
    pub sovereign_owner_shard_0: Address,
    pub sovereign_owner_shard_1: Address,
    pub sovereign_owner_shard_2: Address,
    pub bridge_service_shard_0: Address,
    pub bridge_service_shard_1: Address,
    pub bridge_service_shard_2: Address,
    pub user_address: Address,
    pub state: State,
}
impl CommonInteractorTrait for SovereignForgeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn bridge_owner_shard_0(&self) -> &Address {
        &self.bridge_owner_shard_0
    }

    fn bridge_owner_shard_1(&self) -> &Address {
        &self.bridge_owner_shard_1
    }

    fn bridge_owner_shard_2(&self) -> &Address {
        &self.bridge_owner_shard_2
    }

    fn sovereign_owner_shard_0(&self) -> &Address {
        &self.sovereign_owner_shard_0
    }

    fn sovereign_owner_shard_1(&self) -> &Address {
        &self.sovereign_owner_shard_1
    }

    fn sovereign_owner_shard_2(&self) -> &Address {
        &self.sovereign_owner_shard_2
    }

    fn bridge_service_shard_0(&self) -> &Address {
        &self.bridge_service_shard_0
    }

    fn bridge_service_shard_1(&self) -> &Address {
        &self.bridge_service_shard_1
    }

    fn bridge_service_shard_2(&self) -> &Address {
        &self.bridge_service_shard_2
    }

    fn user_address(&self) -> &Address {
        &self.user_address
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }
}
impl SovereignForgeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config.clone()).await;

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

        let shard_0_wallet = Wallet::from_pem_file("wallets/shard-0-wallet.pem")
            .expect(FAILED_TO_LOAD_WALLET_SHARD_0);

        let bridge_owner_shard_0 = interactor.register_wallet(test_wallets::bob()).await;
        let bridge_owner_shard_1 = interactor.register_wallet(test_wallets::alice()).await;
        let bridge_owner_shard_2 = interactor.register_wallet(test_wallets::carol()).await;
        let sovereign_owner_shard_0 = interactor.register_wallet(test_wallets::mike()).await;
        let sovereign_owner_shard_1 = interactor.register_wallet(test_wallets::frank()).await;
        let sovereign_owner_shard_2 = interactor.register_wallet(test_wallets::heidi()).await;
        let bridge_service_shard_0 = interactor.register_wallet(shard_0_wallet).await;
        let bridge_service_shard_1 = interactor.register_wallet(test_wallets::dan()).await;
        let bridge_service_shard_2 = interactor.register_wallet(test_wallets::judy()).await;
        let user_address = interactor.register_wallet(test_wallets::grace()).await; //shard 1

        interactor.generate_blocks_until_epoch(1).await.unwrap();

        SovereignForgeInteract {
            interactor,
            bridge_owner_shard_0,
            bridge_owner_shard_1,
            bridge_owner_shard_2,
            sovereign_owner_shard_0,
            sovereign_owner_shard_1,
            sovereign_owner_shard_2,
            bridge_service_shard_0,
            bridge_service_shard_1,
            bridge_service_shard_2,
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

        let expected_tokens_wallet = vec![
            self.thousand_tokens(self.state.get_first_token_id_string()),
            self.thousand_tokens(self.state.get_second_token_id_string()),
            self.thousand_tokens(self.state.get_fee_token_id_string()),
            self.one_token(self.state.get_nft_token_id_string()),
            self.thousand_tokens(self.state.get_meta_esdt_token_id_string()),
            self.one_token(self.state.get_dynamic_nft_token_id_string()),
            self.thousand_tokens(self.state.get_sft_token_id_string()),
            self.thousand_tokens(self.state.get_dynamic_meta_esdt_token_id_string()),
            self.thousand_tokens(self.state.get_dynamic_sft_token_id_string()),
        ];
        self.state.set_initial_balance(
            Bech32Address::from(self.user_address().clone()),
            expected_tokens_wallet,
        );
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
}
