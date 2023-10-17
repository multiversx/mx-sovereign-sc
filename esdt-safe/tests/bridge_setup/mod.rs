use bls_signature::BLS_SIGNATURE_LEN;
use esdt_safe::EsdtSafe;
use multiversx_sc::types::{Address, MultiValueEncoded};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use token_module::TokenModule;
use tx_batch_module::TxBatchModule;

multiversx_sc::derive_imports!();

pub static FUNGIBLE_TOKEN_ID: &[u8] = b"FUNGTOKEN-123456";
pub static NFT_TOKEN_ID: &[u8] = b"NFT-123456";
pub const TOKEN_BALANCE: u64 = 1_000_000_000_000_000_000;
pub static DUMMY_SIG: [u8; BLS_SIGNATURE_LEN] = [0; BLS_SIGNATURE_LEN];

#[derive(TopEncode, TopDecode, PartialEq, Debug)]
pub struct DummyAttributes {
    pub dummy: u8,
}

pub struct BridgeSetup<BridgeBuilder>
where
    BridgeBuilder: 'static + Copy + Fn() -> esdt_safe::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner: Address,
    pub user: Address,
    pub sov_dest_addr: Address,
    pub bridge_wrapper: ContractObjWrapper<esdt_safe::ContractObj<DebugApi>, BridgeBuilder>,
}

impl<BridgeBuilder> BridgeSetup<BridgeBuilder>
where
    BridgeBuilder: 'static + Copy + Fn() -> esdt_safe::ContractObj<DebugApi>,
{
    pub fn new(bridge_builder: BridgeBuilder) -> Self {
        let rust_zero = rust_biguint!(0);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner = b_mock.create_user_account(&rust_zero);
        let user = b_mock.create_user_account(&rust_zero);
        let sov_dest_addr = b_mock.create_user_account(&rust_zero);
        let bridge_wrapper =
            b_mock.create_sc_account(&rust_zero, Some(&owner), bridge_builder, "bridge");

        b_mock.set_esdt_balance(&user, FUNGIBLE_TOKEN_ID, &rust_biguint!(TOKEN_BALANCE));
        b_mock.set_nft_balance(
            &user,
            NFT_TOKEN_ID,
            1,
            &rust_biguint!(TOKEN_BALANCE),
            &DummyAttributes { dummy: 42 },
        );

        b_mock
            .execute_tx(&owner, &bridge_wrapper, &rust_zero, |sc| {
                sc.init(0, MultiValueEncoded::new());
                sc.set_max_tx_batch_size(1);
                sc.set_paused(false);
                sc.add_token_to_whitelist(managed_token_id!(FUNGIBLE_TOKEN_ID));
                sc.add_token_to_whitelist(managed_token_id!(NFT_TOKEN_ID));
            })
            .assert_ok();

        BridgeSetup {
            b_mock,
            owner,
            user,
            sov_dest_addr,
            bridge_wrapper,
        }
    }
}
