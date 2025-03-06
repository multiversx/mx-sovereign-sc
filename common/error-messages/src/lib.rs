#![no_std]

pub const CANNOT_CREATE_TX_WHILE_PAUSE: &[u8] = b"Cannot create transaction while paused";
pub const BRIDGE_ALREADY_DEPLOYED: &[u8] = b"Bridge already deployed";
pub const INVALID_MIN_MAX_VALIDATOR_NUMBER: &[u8] = b"Invalid min/max validator numbers";
pub const INVALID_PAYMENT_AMOUNT: &[u8] = b"Invalid payment amount";
pub const ONLY_DEPLOYED_CONTRACTS_CAN_CALL_ENDPOINT: &[u8] =
    b"Only deployed contracts may call this endpoint";
pub const NOTHING_TO_TRANSFER: &[u8] = b"Nothing to transfer";
pub const TOO_MANY_TOKENS: &[u8] = b"Too many tokens";
pub const TOKEN_ALREADY_REGISTERED: &[u8] = b"This token was already registered";
pub const TOKEN_BLACKLISTED: &[u8] = b"Token is blacklisted";
pub const BANNED_ENDPOINT_NAME: &[u8] = b"Banned endpoint name";
pub const GAS_LIMIT_TOO_HIGH: &[u8] = b"Gas limit too high";
pub const NO_HEADER_VERIFIER_ADDRESS: &[u8] = b"There is no registered Header-Verifier address";
pub const MAX_GAS_LIMIT_PER_TX_EXCEEDED: &[u8] =
    b"The gas limit exceeds the maximum gas per transaction limit";
pub const DEPOSIT_OVER_MAX_AMOUNT: &[u8] = b"Deposit over max amount";
pub const INVALID_CALLER: &[u8] = b"Invalid caller";
pub const SETUP_PHASE_NOT_COMPLETED: &[u8] = b"The setup is not completed";
pub const ONLY_ESDT_SAFE_CALLER: &[u8] = b"Only ESDT Safe call this endpoint";
pub const INVALID_FEE: &[u8] = b"Invalid fee";
pub const INVALID_ESDT_IDENTIFIER: &[u8] = b"Invalid ESDT identifier";
pub const INVALID_WEGLD_USDC_PAIR_ADDRESS: &[u8] = b"Invalid WEGLD-USDC pair address from router";
pub const INVALID_TOKEN_USDC_PAIR_ADDRESS: &[u8] = b"Invalid TOKEN-USDC pair address from router";
pub const INVALID_PERCENTAGE_SUM: &[u8] = b"Invalid percentage sum";
pub const INVALID_TOKEN_PROVIDED_FOR_FEE: &[u8] = b"Invalid token provided for fee";
pub const PAYMENT_DOES_NOT_COVER_FEE: &[u8] = b"Payment does not cover fee";
pub const OUTGOING_TX_HASH_ALREADY_REGISTERED: &[u8] =
    b"The OutGoingTxHash has already been registered";
pub const BLS_SIGNATURE_NOT_VALID: &[u8] = b"BLS signature is not valid";
pub const CURRENT_OPERATION_NOT_REGISTERED: &[u8] = b"The current operation is not registered";
pub const CURRENT_OPERATION_ALREADY_IN_EXECUTION: &[u8] =
    b"The current operation is already in execution";
pub const NO_ESDT_SAFE_ADDRESS: &[u8] = b"There is no registered ESDT address";
pub const HASH_OF_HASHES_DOES_NOT_MATCH: &[u8] =
    b"Hash of all operations doesn't match the hash of transfer data";
pub const ESDT_SAFE_STILL_PAUSED: &[u8] = b"Cannot create transaction while paused";
