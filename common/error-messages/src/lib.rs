#![no_std]

pub const BRIDGE_ALREADY_DEPLOYED: &str = "Bridge already deployed";
pub const INVALID_MIN_MAX_VALIDATOR_NUMBERS: &str = "Invalid min/max validator numbers";
pub const INVALID_PAYMENT_AMOUNT: &str = "Invalid payment amount";
pub const ONLY_DEPLOYED_CONTRACTS_CAN_CALL_ENDPOINT: &str =
    "Only deployed contracts may call this endpoint";
pub const NOTHING_TO_TRANSFER: &str = "Nothing to transfer";
pub const TOO_MANY_TOKENS: &str = "Too many tokens";
pub const TOKEN_ALREADY_REGISTERED: &str = "This token was already registered";
pub const TOKEN_BLACKLISTED: &str = "Token is blacklisted";
pub const BANNED_ENDPOINT_NAME: &str = "Banned endpoint name";
pub const GAS_LIMIT_TOO_HIGH: &str = "Gas limit too high";
pub const NO_HEADER_VERIFIER_ADDRESS: &str = "There is no registered Header-Verifier address";
pub const MAX_GAS_LIMIT_PER_TX_EXCEEDED: &str =
    "The gas limit exceeds the maximum gas per transaction limit";
pub const DEPOSIT_OVER_MAX_AMOUNT: &str = "Deposit over max amount";
pub const INVALID_CALLER: &str = "Invalid caller";
pub const SETUP_PHASE_NOT_COMPLETED: &str = "The setup is not completed";
pub const ONLY_ESDT_SAFE_CALLER: &str = "Only ESDT Safe can call this endpoint";
pub const INVALID_FEE: &str = "Invalid fee";
pub const INVALID_ESDT_IDENTIFIER: &str = "Invalid ESDT identifier";
pub const INVALID_WEGLD_USDC_PAIR_ADDRESS: &str = "Invalid WEGLD-USDC pair address from router";
pub const INVALID_TOKEN_USDC_PAIR_ADDRESS: &str = "Invalid TOKEN-USDC pair address from router";
pub const INVALID_PERCENTAGE_SUM: &str = "Invalid percentage sum";
pub const INVALID_TOKEN_PROVIDED_FOR_FEE: &str = "Invalid token provided for fee";
pub const PAYMENT_DOES_NOT_COVER_FEE: &str = "Payment does not cover fee";
pub const OUTGOING_TX_HASH_ALREADY_REGISTERED: &str =
    "The OutGoingTxHash has already been registered";
pub const BLS_SIGNATURE_NOT_VALID: &str = "BLS signature is not valid";
pub const CURRENT_OPERATION_NOT_REGISTERED: &str = "The current operation is not registered";
pub const CURRENT_OPERATION_ALREADY_IN_EXECUTION: &str =
    "The current operation is already in execution";
pub const NO_ESDT_SAFE_ADDRESS: &str = "There is no registered ESDT address";
pub const HASH_OF_HASHES_DOES_NOT_MATCH: &str =
    "Hash of all operations doesn't match the hash of transfer data";
pub const ESDT_SAFE_STILL_PAUSED: &str = "Cannot create transaction while paused";
pub const INVALID_TYPE: &str = "Invalid type";
pub const INVALID_FEE_TYPE: &str = "Invalid fee type";
pub const INVALID_TOKEN_ID: &str = "Invalid token ID";
pub const INVALID_SC_ADDRESS: &str = "Invalid SC address";
pub const ITEM_NOT_IN_LIST: &str = "Item not found in list";
pub const TOKEN_ID_NO_PREFIX: &str = "Token Id does not have prefix";
pub const TOKEN_NOT_ACCEPTED_AS_FEE: &str = "Token not accepted as fee";
pub const TOKEN_ID_IS_NOT_TRUSTED: &str = "Token is not trusted";
pub const MINT_AND_BURN_ROLES_NOT_FOUND: &str = "This token does not have Mint and Burn roles";
pub const TOKEN_IS_FROM_SOVEREIGN: &str = "Token is from a Sovereign Chain, it cannot be locked";
