use bls_signature::BLS_SIGNATURE_LEN;
use esdt_safe::EsdtSafe;
use fee_market::{
    fee_type::{FeeType, FeeTypeModule},
    FeeMarket,
};
use multiversx_sc::types::{Address, MultiValueEncoded};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use tx_batch_module::TxBatchModule;

multiversx_sc::derive_imports!();

pub static FUNGIBLE_TOKEN_ID: &[u8] = b"FUNGTOKEN-123456";
pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-123456";
pub static USDC_TOKEN_ID: &[u8] = b"USDC-123456";
pub static NFT_TOKEN_ID: &[u8] = b"NFT-123456";
pub const TOKEN_BALANCE: u64 = 1_000_000_000_000_000_000;
pub static DUMMY_SIG: [u8; BLS_SIGNATURE_LEN] = [0; BLS_SIGNATURE_LEN];
pub static FEE_TOKEN_ID: &[u8] = b"FEE-123456";

#[derive(TopEncode, TopDecode, PartialEq, Debug)]
pub struct DummyAttributes {
    pub dummy: u8,
}

pub struct BridgeSetup<BridgeBuilder, FeeMarketBuilder>
where
    BridgeBuilder: 'static + Copy + Fn() -> esdt_safe::ContractObj<DebugApi>,
    FeeMarketBuilder: 'static + Copy + Fn() -> fee_market::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner: Address,
    pub user: Address,
    pub sov_dest_addr: Address,
    pub bridge_wrapper: ContractObjWrapper<esdt_safe::ContractObj<DebugApi>, BridgeBuilder>,
    pub fee_market_wrapper: ContractObjWrapper<fee_market::ContractObj<DebugApi>, FeeMarketBuilder>,
}

impl<BridgeBuilder, FeeMarketBuilder> BridgeSetup<BridgeBuilder, FeeMarketBuilder>
where
    BridgeBuilder: 'static + Copy + Fn() -> esdt_safe::ContractObj<DebugApi>,
    FeeMarketBuilder: 'static + Copy + Fn() -> fee_market::ContractObj<DebugApi>,
{
    pub fn new(bridge_builder: BridgeBuilder, fee_market_builder: FeeMarketBuilder, is_sovereign_chain: bool) -> Self {
        let rust_zero = rust_biguint!(0);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner = b_mock.create_user_account(&rust_zero);
        let user = b_mock.create_user_account(&rust_zero);
        let sov_dest_addr = b_mock.create_user_account(&rust_zero);
        let bridge_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&owner), bridge_builder, "bridge");
        let fee_market_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&owner), fee_market_builder, "fee_market");

        b_mock.set_esdt_balance(&user, FUNGIBLE_TOKEN_ID, &rust_biguint!(TOKEN_BALANCE));
        b_mock.set_esdt_balance(&user, FEE_TOKEN_ID, &rust_biguint!(TOKEN_BALANCE));
        b_mock.set_nft_balance(
            &user,
            NFT_TOKEN_ID,
            1,
            &rust_biguint!(TOKEN_BALANCE),
            &DummyAttributes { dummy: 42 },
        );

        b_mock
            .execute_tx(&owner, &bridge_wrapper, &rust_zero, |sc| {
                sc.init(is_sovereign_chain, 0, managed_address!(&owner), MultiValueEncoded::new());
                sc.set_fee_market_address(managed_address!(fee_market_wrapper.address_ref()));
                sc.set_max_tx_batch_size(1);
                sc.set_paused(false);
            })
            .assert_ok();

        b_mock
            .execute_tx(&owner, &fee_market_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_address!(bridge_wrapper.address_ref()),
                    managed_address!(bridge_wrapper.address_ref()), // unused
                    WEGLD_TOKEN_ID.into(),
                    USDC_TOKEN_ID.into()
                );
                sc.add_fee(
                    managed_token_id!(FEE_TOKEN_ID),
                    FeeType::Fixed {
                        token: managed_token_id!(FEE_TOKEN_ID),
                        per_transfer: managed_biguint!(100),
                        per_gas: managed_biguint!(1),
                    },
                )
            })
            .assert_ok();

        BridgeSetup {
            b_mock,
            owner,
            user,
            sov_dest_addr,
            bridge_wrapper,
            fee_market_wrapper,
        }
    }
}
