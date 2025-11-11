#![no_std]

macro_rules! error_messages {
    ($($name:ident => $msg:literal),* $(,)?) => {
        $(pub const $name: &str = $msg;)*
        pub const ALL_ERROR_MESSAGES: &[&str] = &[$($msg),*];
    };
}

error_messages! {
    ACTION_IS_NOT_ALLOWED => "action is not allowed",
    ADDITIONAL_STAKE_NOT_REQUIRED => "Additional stake was provided but is not required",
    ADDITIONAL_STAKE_ZERO_VALUE => "Additional stake cannot be a zero value",
    ADDRESS_NOT_VALID_SC_ADDRESS => "The given address is not a valid SC address",
    AMOUNT_IS_TOO_LARGE => "Amount is too large",
    BANNED_ENDPOINT_NAME => "Banned endpoint name",
    BLS_KEY_NOT_REGISTERED => "BLS key not registered",
    BURN_ESDT_FAILED => "Failed to burn ESDT:",
    BURN_MECHANISM_NON_ESDT_TOKENS => "Non-ESDT tokens can not have a burn mechanism",
    CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN => "The current caller has not deployed any Sovereign Chain",
    CALLER_IS_BLACKLISTED => "Caller is blacklisted",
    CALLER_NOT_FROM_CURRENT_SOVEREIGN => "Caller is not from the current Sovereign-Chain",
    CALLER_NOT_OWNER => "Endpoint can only be called by owner",
    CANNOT_REGISTER_TOKEN => "Cannot register token",
    CHAIN_CONFIG_ALREADY_DEPLOYED => "The Chain-Config contract is already deployed",
    CHAIN_CONFIG_NOT_DEPLOYED => "The Chain-Config SC is not deployed",
    CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE =>
        "The Chain-Config SC setup phase is not completed",
    CHAIN_FACTORY_ADDRESS_NOT_IN_EXPECTED_SHARD =>
        "This Chain-Factory SC is not deployed in the specified shard ID",
    CHAIN_ID_ALREADY_IN_USE => "This chain ID is already used",
    CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC => "Chain ID is not lowercase alphanumeric",
    CREATE_ESDT_FAILED => "Failed to create ESDT:",
    CURRENT_OPERATION_ALREADY_IN_EXECUTION => "The current operation is already in execution",
    CURRENT_OPERATION_NOT_REGISTERED => "The current operation is not registered",
    DEPOSIT_AMOUNT_NOT_ENOUGH => "Deposit amount is less than the operation amount",
    DEPOSIT_OVER_MAX_AMOUNT => "Deposit over max amount",
    DEPLOY_COST_NOT_ENOUGH => "The given deploy cost is not equal to the standard amount",
    DUPLICATE_ADDITIONAL_STAKE_TOKEN_ID =>
        "Duplicate additional stake token identifier",
    EGLD_TOKEN_IDENTIFIER_EXPECTED =>
        "The token identifier should be the EGLD token identifier",
    ERROR_AT_GENERATING_OPERATION_HASH => "Error at generating operation hash",
    ERR_EMPTY_PAYMENTS => "No payments",
    ESDT_SAFE_ADDRESS_NOT_SET => "The ESDT-Safe address is not set",
    ESDT_SAFE_ALREADY_DEPLOYED => "The ESDT-Safe SC is already deployed",
    ESDT_SAFE_NOT_DEPLOYED =>
        "The ESDT-Safe SC is not deployed, you skipped the second phase",
    ESDT_SAFE_STILL_PAUSED => "Cannot create transaction while paused",
    EXPECTED_MAPPED_TOKEN => "Expected mapped token, got None",
    FAILED_TO_PARSE_AS_NUMBER => "Failed to parse actual amount as number",
    FAILED_TO_REGISTER_SOVEREIGN_TOKEN => "Failed to register sovereign token",
    FEE_MARKET_ALREADY_DEPLOYED => "The Fee-Market SC is already deployed",
    FEE_MARKET_NOT_DEPLOYED => "The Fee-Market SC is not deployed",
    FEE_MARKET_NOT_SET => "There is no Fee-Market address set",
    GAS_LIMIT_TOO_HIGH => "Gas limit too high",
    HASH_OF_HASHES_DOES_NOT_MATCH =>
        "Hash of all operations doesn't match the hash of transfer data",
    HEADER_VERIFIER_ALREADY_DEPLOYED =>
        "The Header-Verifier contract is already deployed",
    HEADER_VERIFIER_NOT_DEPLOYED =>
        "The Header-Verifier SC is not deployed, you skipped the fourth phase",
    INCORRECT_DEPOSIT_AMOUNT => "Incorrect deposit amount",
    INCORRECT_OPERATION_NONCE => "The operation nonce is incorrect",
    INVALID_ADDITIONAL_STAKE => "Invalid additional stake sent",
    INVALID_BLS_KEY_FOR_CALLER => "Invalid BLS key for caller",
    INVALID_BLS_KEY_PROVIDED => "Invalid BLS key has been provided",
    INVALID_CALLER => "Invalid caller",
    INVALID_CHAIN_ID => "Invalid chain ID",
    INVALID_EGLD_STAKE => "Invalid EGLD stake sent",
    INVALID_EPOCH => "Cannot change the validator set for the genesis epoch",
    INVALID_FEE => "Invalid fee",
    INVALID_FEE_TYPE => "Invalid fee type",
    INVALID_FUNCTION_NOT_FOUND => "invalid function (not found)",
    INVALID_MIN_MAX_VALIDATOR_NUMBERS => "Invalid validator range",
    INVALID_PERCENTAGE_SUM => "Invalid percentage sum",
    INVALID_PREFIX => "The sovereign prefix should be between 1 and 4 characters long",
    INVALID_PREFIX_FOR_REGISTER =>
        "Provided sovereign token identifier has invalid prefix",
    INVALID_SC_ADDRESS => "Invalid SC address",
    INVALID_TOKEN_ID => "Invalid token ID",
    INVALID_TOKEN_PROVIDED_FOR_FEE => "Invalid token provided for fee",
    INVALID_TYPE => "Invalid type",
    INVALID_VALIDATOR_DATA => "Invalid validator data has been provided",
    ISSUE_COST_NOT_COVERED => "Native token issue cost is not covered",
    ITEM_NOT_IN_LIST => "Item not found in list",
    LOCK_MECHANISM_NON_ESDT => "Non-ESDT tokens can not have a lock mechanism",
    MAX_GAS_LIMIT_PER_TX_EXCEEDED =>
        "The gas limit exceeds the maximum gas per transaction limit",
    MINT_AND_BURN_ROLES_NOT_FOUND => "This token does not have Mint and Burn roles",
    MINT_ESDT_FAILED => "Failed to mint ESDT:",
    NATIVE_TOKEN_ALREADY_REGISTERED => "Native token was already registered",
    NATIVE_TOKEN_NOT_REGISTERED => "There is no native token registered",
    NO_ADDRESSES_AVAILABLE => "No addresses available",
    NO_ESDT_SAFE_ADDRESS => "There is no registered ESDT address",
    NO_KNOWN_CHAIN_CONFIG_SC => "No known Chain Config SC contract, deploy first",
    NO_KNOWN_CHAIN_FACTORY_IN_THE_SPECIFIED_SHARD =>
        "No chain factory address found for the specified shard",
    NO_KNOWN_CHAIN_FACTORY_SC => "No known Chain Factory SC, deploy first",
    NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID => "No known Dynamic Meta ESDT token ID",
    NO_KNOWN_DYNAMIC_NFT_TOKEN_ID => "No known Dynamic NFT token ID",
    NO_KNOWN_DYNAMIC_SFT_TOKEN_ID => "No known Dynamic SFT token ID",
    NO_KNOWN_FEE_MARKET => "No known Fee Market contract, deploy first",
    NO_KNOWN_FEE_TOKEN => "No known fee token, register first",
    NO_KNOWN_FIRST_TOKEN => "No known first token, register first",
    NO_KNOWN_FUNGIBLE_TOKEN => "No known fungible token, register first",
    NO_KNOWN_HEADER_VERIFIER => "No known Header Verifier contract, deploy first",
    NO_KNOWN_META_ESDT_TOKEN => "No known Meta ESDT token ID",
    NO_KNOWN_MVX_ESDT_SAFE => "No known MVX ESDT Safe contract, deploy first",
    NO_KNOWN_NFT_TOKEN => "No known NFT token, register first",
    NO_KNOWN_SFT_TOKEN => "No known SFT token, register first",
    NO_KNOWN_SOVEREIGN_FORGE_SC => "No known Sovereign Forge SC, deploy first",
    NO_KNOWN_TESTING_SC => "No known Testing SC contract, deploy first",
    NO_KNOWN_TRUSTED_TOKEN => "No known trusted token, register first",
    NO_VALIDATORS_FOR_GIVEN_EPOCH =>
        "There are no registered validators for the given epoch",
    NO_VALIDATORS_FOR_PREVIOUS_EPOCH =>
        "There are no registered validators for the previous epoch",
    NOT_ENOUGH_EGLD_FOR_REGISTER => "Not enough EGLD for registering a new token",
    NOT_ENOUGH_VALIDATORS => "Not enough validators registered",
    NOTHING_TO_TRANSFER => "Nothing to transfer",
    ONLY_ESDT_SAFE_CALLER => "Only ESDT Safe can call this endpoint",
    OUTGOING_TX_HASH_ALREADY_REGISTERED =>
        "The OutGoingTxHash has already been registered",
    PAYMENT_DOES_NOT_COVER_FEE => "Payment does not cover fee",
    REGISTRATIONS_DISABLED_GENESIS_PHASE =>
        "Registrations are disabled after genesis phase",
    SETUP_PHASE_ALREADY_COMPLETED => "The setup is completed",
    SETUP_PHASE_NOT_COMPLETED => "The setup is not completed",
    SOVEREIGN_SETUP_PHASE_ALREADY_COMPLETED =>
        "This Sovereign-Chain's setup phase is already completed",
    TOKEN_ALREADY_REGISTERED => "This token was already registered",
    TOKEN_ALREADY_REGISTERED_WITH_BURN_MECHANISM =>
        "Token already registered in burn mechanism",
    TOKEN_BLACKLISTED => "Token is blacklisted",
    TOKEN_ID_IS_NOT_TRUSTED => "Token is not trusted",
    TOKEN_ID_NO_PREFIX => "Token Id does not have prefix",
    TOKEN_NOT_ACCEPTED_AS_FEE => "Token not accepted as fee",
    TOKEN_NOT_REGISTERED_WITH_BURN_MECHANISM =>
        "Token not registered in burn mechanism",
    TOO_MANY_TOKENS => "Too many tokens",
    VALIDATOR_ALREADY_REGISTERED => "Validator already registered",
    VALIDATOR_ID_NOT_REGISTERED => "Provided validator id is not registered",
    VALIDATOR_NOT_REGISTERED => "Validator not registered",
    VALIDATOR_RANGE_EXCEEDED => "Validator range exceeded",
    VALIDATORS_ALREADY_REGISTERED_IN_EPOCH =>
        "There already is a validator set registered for this epoch",
}
