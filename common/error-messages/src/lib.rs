#![no_std]

pub const ACTION_IS_NOT_ALLOWED: &str = "action is not allowed";
pub const ADDRESS_NOT_VALID_SC_ADDRESS: &str = "The given address is not a valid SC address";
pub const BANNED_ENDPOINT_NAME: &str = "Banned endpoint name";
pub const BLS_SIGNATURE_NOT_VALID: &str = "BLS signature is not valid";
pub const BRIDGE_ALREADY_DEPLOYED: &str = "Bridge already deployed";
pub const CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN: &str =
    "The current caller has not deployed any Sovereign Chain";
pub const CALLER_NOT_FROM_CURRENT_SOVEREIGN: &str =
    "Caller is not from the current Sovereign-Chain";
pub const CALLER_IS_NOT_WHITELISTED: &str = "Caller is not whitelisted";
pub const CANNOT_REGISTER_TOKEN: &str = "Cannot register token";
pub const CANNOT_TRANSFER_WHILE_PAUSED: &str = "Cannot transfer while paused";
pub const CHAIN_CONFIG_ALREADY_DEPLOYED: &str = "The Chain-Config contract is already deployed";
pub const CHAIN_CONFIG_NOT_DEPLOYED: &str = "The Chain-Config SC is not deployed";
pub const CHAIN_ID_ALREADY_IN_USE: &str = "This chain ID is already used";
pub const CHAIN_ID_NOT_FOUR_CHAR_LONG: &str = "Chain ID length must be four characters";
pub const CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC: &str = "Chain ID is not lowercase alphanumeric";
pub const CURRENT_OPERATION_ALREADY_IN_EXECUTION: &str =
    "The current operation is already in execution";
pub const CURRENT_OPERATION_NOT_REGISTERED: &str = "The current operation is not registered";
pub const DEPLOY_COST_IS_ZERO: &str = "The deploy cost can't be a 0 value";
pub const DEPLOY_COST_NOT_ENOUGH: &str =
    "The given deploy cost is not equal to the standard amount";
pub const DEPOSIT_AMOUNT_SMALLER_THAN_PAYMENT_AMOUNT: &str =
    "The deposit amount should not be less than the payment amount";
pub const DEPOSIT_OVER_MAX_AMOUNT: &str = "Deposit over max amount";
pub const ENDPOINT_CAN_ONLY_BE_CALLED_BY_ADMIN: &str = "Endpoint can only be called by admins";
pub const ERR_EMPTY_PAYMENTS: &str = "No payments";
pub const ESDT_SAFE_ADDRESS_NOT_SET: &str = "The ESDT-Safe address is not set";
pub const ESDT_SAFE_ALREADY_DEPLOYED: &str = "The ESDT-Safe SC is already deployed";
pub const ESDT_SAFE_CONFIG_NOT_SET: &str = "There is no config set for this contract";
pub const ESDT_SAFE_NOT_DEPLOYED: &str =
    "The ESDT-Safe SC is not deployed, you skipped the third phase";
pub const ESDT_SAFE_STILL_PAUSED: &str = "Cannot create transaction while paused";
pub const FEE_MARKET_ALREADY_DEPLOYED: &str = "The Fee-Market SC is already deployed";
pub const FEE_MARKET_NOT_DEPLOYED: &str = "The Fee-Market SC is not deployed";
pub const GAS_LIMIT_TOO_HIGH: &str = "Gas limit too high";
pub const HASH_OF_HASHES_DOES_NOT_MATCH: &str =
    "Hash of all operations doesn't match the hash of transfer data";
pub const HEADER_VERIFIER_ADDRESS_NOT_SET: &str = "The Header-Verifier address was not set";
pub const HEADER_VERIFIER_ALREADY_DEPLOYED: &str =
    "The Header-Verifier contract is already deployed";
pub const HEADER_VERIFIER_NOT_DEPLOYED: &str =
    "The Header-Verifier SC is not deployed, you skipped the second phase";
pub const INSUFFICIENT_FUNDS: &str = "insufficient funds";
pub const INVALID_AGGREGATOR_VALUE: &str = "Invalid aggregator value";
pub const INVALID_CALLER: &str = "Invalid caller";
pub const INVALID_ESDT_IDENTIFIER: &str = "Invalid ESDT identifier";
pub const INVALID_FEE: &str = "Invalid fee";
pub const INVALID_FEE_TYPE: &str = "Invalid fee type";
pub const INVALID_METHOD_TO_CALL_IN_CURRENT_CHAIN: &str = "Invalid method to call in current chain";
pub const INVALID_MIN_MAX_VALIDATOR_NUMBERS: &str = "Invalid min/max validator numbers";
pub const INVALID_PAYMENT_AMOUNT: &str = "Invalid payment amount";
pub const INVALID_PERCENTAGE_SUM: &str = "Invalid percentage sum";
pub const INVALID_SC_ADDRESS: &str = "Invalid SC address";
pub const INVALID_TOKEN_ID: &str = "Invalid token ID";
pub const INVALID_TOKEN_PROVIDED_FOR_FEE: &str = "Invalid token provided for fee";
pub const INVALID_TOKEN_USDC_PAIR_ADDRESS: &str = "Invalid TOKEN-USDC pair address from router";
pub const INVALID_TYPE: &str = "Invalid type";
pub const INVALID_VALIDATOR_SET_LENGTH: &str =
    "The current validator set length doesn't meet the Sovereign's requirements";
pub const INVALID_WEGLD_USDC_PAIR_ADDRESS: &str = "Invalid WEGLD-USDC pair address from router";
pub const ITEM_NOT_IN_LIST: &str = "Item not found in list";
pub const MAX_GAS_LIMIT_PER_TX_EXCEEDED: &str =
    "The gas limit exceeds the maximum gas per transaction limit";
pub const MINT_AND_BURN_ROLES_NOT_FOUND: &str = "This token does not have Mint and Burn roles";
pub const NATIVE_TOKEN_ALREADY_REGISTERED: &str = "Native token was already registered";
pub const NATIVE_TOKEN_NOT_REGISTERED: &str = "There is no native token registered";
pub const NO_ESDT_SAFE_ADDRESS: &str = "There is no registered ESDT address";
pub const NO_HEADER_VERIFIER_ADDRESS: &str = "There is no registered Header-Verifier address";
pub const NOT_ENOUGH_WEGLD_AMOUNT: &str = "WEGLD fee amount is not met";
pub const NOTHING_TO_TRANSFER: &str = "Nothing to transfer";
pub const ONLY_DEPLOYED_CONTRACTS_CAN_CALL_ENDPOINT: &str =
    "Only deployed contracts may call this endpoint";
pub const ONLY_ESDT_SAFE_CALLER: &str = "Only ESDT Safe can call this endpoint";
pub const ONLY_WEGLD_IS_ACCEPTED_AS_REGISTER_FEE: &str =
    "WEGLD is the only token accepted as register fee";
pub const OUTGOING_TX_HASH_ALREADY_REGISTERED: &str =
    "The OutGoingTxHash has already been registered";
pub const PAYMENT_DOES_NOT_COVER_FEE: &str = "Payment does not cover fee";
pub const SETUP_PHASE_ALREADY_COMPLETED: &str = "The setup is completed";
pub const SETUP_PHASE_NOT_COMPLETED: &str = "The setup is not completed";
pub const SOVEREIGN_SETUP_PHASE_ALREADY_COMPLETED: &str =
    "This Sovereign-Chain's setup phase is already completed";
pub const TOKEN_ALREADY_REGISTERED: &str = "This token was already registered";
pub const TOKEN_BLACKLISTED: &str = "Token is blacklisted";
pub const TOKEN_ID_IS_NOT_TRUSTED: &str = "Token is not trusted";
pub const TOKEN_ID_NO_PREFIX: &str = "Token Id does not have prefix";
pub const TOKEN_IS_FROM_SOVEREIGN: &str = "Token is from a Sovereign Chain, it cannot be locked";
pub const TOKEN_NOT_ACCEPTED_AS_FEE: &str = "Token not accepted as fee";
pub const TOO_MANY_TOKENS: &str = "Too many tokens";
