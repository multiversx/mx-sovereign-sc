#![no_std]

pub const ACTION_IS_NOT_ALLOWED: &str = "action is not allowed";
pub const ADDRESS_NOT_VALID_SC_ADDRESS: &str = "The given address is not a valid SC address";
pub const AMOUNT_IS_TOO_LARGE: &str = "Amount is too large";
pub const BANNED_ENDPOINT_NAME: &str = "Banned endpoint name";
pub const BLS_SIGNATURE_NOT_VALID: &str = "BLS signature is not valid";
pub const BRIDGE_ALREADY_DEPLOYED: &str = "Bridge already deployed";
pub const BURN_NON_ESDT_TOKENS: &str = "Burning non-ESDT tokens is not allowed";
pub const BURN_MECHANISM_NON_ESDT_TOKENS: &str = "Non-ESDT tokens can not have a burn mechanism";
pub const CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN: &str =
    "The current caller has not deployed any Sovereign Chain";
pub const CALLER_NOT_FROM_CURRENT_SOVEREIGN: &str =
    "Caller is not from the current Sovereign-Chain";
pub const CALLER_NOT_OWNER: &str = "Endpoint can only be called by owner";
pub const CALLER_IS_NOT_WHITELISTED: &str = "Caller is not whitelisted";
pub const CANNOT_REGISTER_TOKEN: &str = "Cannot register token";
pub const COULD_NOT_RETRIEVE_SOVEREIGN_CONFIG: &str = "Error at retrieving Sovereign Config";
pub const CANNOT_TRANSFER_WHILE_PAUSED: &str = "Cannot transfer while paused";
pub const CHAIN_CONFIG_ALREADY_DEPLOYED: &str = "The Chain-Config contract is already deployed";
pub const CHAIN_CONFIG_NOT_DEPLOYED: &str = "The Chain-Config SC is not deployed";
pub const CHAIN_ID_ALREADY_IN_USE: &str = "This chain ID is already used";
pub const INVALID_CHAIN_ID: &str = "Invalid chain ID";
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
pub const EMPTY_EXPECTED_LOG: &str = "Expected log string cannot be empty";
pub const ENDPOINT_CAN_ONLY_BE_CALLED_BY_ADMIN: &str = "Endpoint can only be called by admins";
pub const ERR_EMPTY_PAYMENTS: &str = "No payments";
pub const ESDT_SAFE_ADDRESS_NOT_SET: &str = "The ESDT-Safe address is not set";
pub const ESDT_SAFE_ALREADY_DEPLOYED: &str = "The ESDT-Safe SC is already deployed";
pub const ESDT_SAFE_CONFIG_NOT_SET: &str = "There is no config set for this contract";
pub const ESDT_SAFE_NOT_DEPLOYED: &str =
    "The ESDT-Safe SC is not deployed, you skipped the second phase";
pub const ESDT_SAFE_STILL_PAUSED: &str = "Cannot create transaction while paused";
pub const EXPECTED_MAPPED_TOKEN: &str = "Expected mapped token, got None";
pub const FAILED_TO_PARSE_AS_NUMBER: &str = "Failed to parse actual amount as number";
pub const FAILED_TO_REGISTER_SOVEREIGN_TOKEN: &str = "Failed to register sovereign token";
pub const FEE_MARKET_ALREADY_DEPLOYED: &str = "The Fee-Market SC is already deployed";
pub const FEE_MARKET_NOT_DEPLOYED: &str = "The Fee-Market SC is not deployed";
pub const FEE_MARKET_NOT_SET: &str = "There is no Fee-Market address set";
pub const GAS_LIMIT_TOO_HIGH: &str = "Gas limit too high";
pub const HASH_OF_HASHES_DOES_NOT_MATCH: &str =
    "Hash of all operations doesn't match the hash of transfer data";
pub const HEADER_VERIFIER_ADDRESS_NOT_SET: &str = "The Header-Verifier address was not set";
pub const HEADER_VERIFIER_ALREADY_DEPLOYED: &str =
    "The Header-Verifier contract is already deployed";
pub const HEADER_VERIFIER_NOT_DEPLOYED: &str =
    "The Header-Verifier SC is not deployed, you skipped the fourth phase";
pub const INSUFFICIENT_FUNDS: &str = "insufficient funds";
pub const INVALID_AGGREGATOR_VALUE: &str = "Invalid aggregator value";
pub const INVALID_CALLER: &str = "Invalid caller";
pub const INVALID_ESDT_IDENTIFIER: &str = "Invalid ESDT identifier";
pub const INVALID_FEE: &str = "Invalid fee";
pub const INVALID_FEE_TYPE: &str = "Invalid fee type";
pub const INVALID_METHOD_TO_CALL_IN_CURRENT_CHAIN: &str = "Invalid method to call in current chain";
pub const INVALID_MIN_MAX_VALIDATOR_NUMBERS: &str = "Invalid validator range";
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
pub const LOCK_MECHANISM_NON_ESDT: &str = "Non-ESDT tokens can not have a lock mechanism";
pub const MAX_GAS_LIMIT_PER_TX_EXCEEDED: &str =
    "The gas limit exceeds the maximum gas per transaction limit";
pub const MINT_AND_BURN_ROLES_NOT_FOUND: &str = "This token does not have Mint and Burn roles";
pub const MINT_NON_ESDT_TOKENS: &str = "Non-ESDT tokens can not be minted";
pub const NATIVE_TOKEN_ALREADY_REGISTERED: &str = "Native token was already registered";
pub const NATIVE_TOKEN_NOT_REGISTERED: &str = "There is no native token registered";
pub const NO_ADDRESSES_AVAILABLE: &str = "No addresses available";
pub const NO_ESDT_SAFE_ADDRESS: &str = "There is no registered ESDT address";
pub const NO_HEADER_VERIFIER_ADDRESS: &str = "There is no registered Header-Verifier address";
pub const NO_KNOWN_CHAIN_CONFIG_SC: &str = "No known Chain Config SC contract, deploy first";
pub const NO_KNOWN_CHAIN_FACTORY_IN_THE_SPECIFIED_SHARD: &str =
    "No chain factory address found for the specified shard";
pub const NO_KNOWN_CHAIN_FACTORY_SC: &str = "No known Chain Factory SC, deploy first";
pub const NO_KNOWN_DYNAMIC_META_ESDT_TOKEN_ID: &str = "No known Dynamic Meta ESDT token ID";
pub const NO_KNOWN_DYNAMIC_NFT_TOKEN_ID: &str = "No known Dynamic NFT token ID";
pub const NO_KNOWN_DYNAMIC_SFT_TOKEN_ID: &str = "No known Dynamic SFT token ID";
pub const NO_KNOWN_FIRST_TOKEN: &str = "No known first token, register first";
pub const NO_KNOWN_FEE_MARKET: &str = "No known Fee Market contract, deploy first";
pub const NO_KNOWN_FEE_TOKEN: &str = "No known fee token, register first";
pub const NO_KNOWN_TRUSTED_TOKEN: &str = "No known trusted token, register first";
pub const NO_KNOWN_FUNGIBLE_TOKEN: &str = "No known fungible token, register first";
pub const NO_KNOWN_MVX_ESDT_SAFE: &str = "No known MVX ESDT Safe contract, deploy first";
pub const NO_KNOWN_HEADER_VERIFIER: &str = "No known Header Verifier contract, deploy first";
pub const NO_KNOWN_SECOND_TOKEN: &str = "No known second token, register first";
pub const NO_KNOWN_SOVEREIGN_FORGE_SC: &str = "No known Sovereign Forge SC, deploy first";
pub const NO_KNOWN_META_ESDT_TOKEN: &str = "No known Meta ESDT token ID";
pub const NO_KNOWN_NFT_TOKEN: &str = "No known NFT token, register first";
pub const NO_KNOWN_SOV_TO_MVX_TOKEN: &str = "No known Sovereign to MVX token ID";
pub const NO_KNOWN_SOV_REGISTRAR: &str =
    "No known Sovereign Registrar smart contract, deploy first";
pub const NO_KNOWN_SFT_TOKEN: &str = "No known SFT token, register first";
pub const NO_KNOWN_TESTING_SC: &str = "No known Testing SC contract, deploy first";
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
pub const INVALID_PREFIX: &str = "The sovereign prefix should be between 1 and 4 characters long";
pub const INVALID_PREFIX_FOR_REGISTER: &str =
    "Provided sovereign token identifier has invalid prefix";
pub const TOKEN_IS_FROM_SOVEREIGN: &str = "Token is from a Sovereign Chain, it cannot be locked";
pub const TOKEN_NOT_ACCEPTED_AS_FEE: &str = "Token not accepted as fee";
pub const TOO_MANY_TOKENS: &str = "Too many tokens";
pub const ERROR_AT_GENERATING_OPERATION_HASH: &str = "Error at generating operation hash";
pub const NOT_ENOUGH_EGLD_FOR_REGISTER: &str = "Not enough EGLD for registering a new token";
pub const VALIDATOR_RANGE_EXCEEDED: &str = "Validator range exceeded";
pub const NOT_ENOUGH_VALIDATORS: &str = "Not enough validators registered";
pub const VALIDATOR_NOT_REGISTERED: &str = "Validator not registered";
pub const VALIDATOR_ALREADY_REGISTERED: &str = "Validator already registered";
pub const BLS_KEY_NOT_REGISTERED: &str = "BLS key not registered";
pub const MIN_NUMBER_OF_SIGNATURE_NOT_MET: &str = "Minimum number of signatures was not met";
pub const VALIDATORS_ALREADY_REGISTERED_IN_EPOCH: &str =
    "There already is a validator set registered for this epoch";
pub const BITMAP_LEN_DOES_NOT_MATCH_BLS_KEY_LEN: &str =
    "Bitmap length does not match BLS keys length";
pub const INVALID_ADDITIONAL_STAKE: &str = "Invalid additional stake sent";
pub const INVALID_EGLD_STAKE: &str = "Invalid EGLD stake sent";
pub const REGISTRATION_DISABLED: &str = "Registration is disabled";
pub const INVALID_REGISTRATION_STATUS: &str = "Registration status can only be 0 or 1";
pub const EMPTY_ADDITIONAL_STAKE: &str = "Additional stake was sent as an empty array";
pub const ADDITIONAL_STAKE_ZERO_VALUE: &str = "Additional stake cannot be a zero value";
pub const ADDITIONAL_STAKE_NOT_REQUIRED: &str = "Additional stake was provided but is not required";
pub const INVALID_BLS_KEY_FOR_CALLER: &str = "Invalid BLS key for caller";
pub const GENESIS_VALIDATORS_ALREADY_SET: &str = "Genesis validators were already set";
pub const GENESIS_VALIDATORS_NOT_SET: &str = "Genesis validators were not set";
pub const INVALID_EPOCH: &str = "Cannot change the validator set for the genesis epoch";
pub const CALLER_NOT_CHAIN_CONFIG: &str = "Only Chain-Config SC can call this endpoint";
pub const CHAIN_CONFIG_SETUP_PHASE_NOT_COMPLETE: &str =
    "The Chain-Config SC setup phase is not completed";
pub const DEPOSIT_AMOUNT_NOT_ENOUGH: &str = "Deposit amount is less than the operation amount";
pub const CHAIN_FACTORY_ADDRESS_NOT_IN_EXPECTED_SHARD: &str =
    "This Chain-Factory SC is not deployed in the specified shard ID";
pub const INVALID_BLS_KEY_PROVIDED: &str = "Invalid BLS key has been provided";
pub const REGISTRATIONS_DISABLED_GENESIS_PHASE: &str =
    "Registrations are disabled after genesis phase";
pub const VALIDATOR_ID_NOT_REGISTERED: &str = "Provided validator id is not registered";
pub const INVALID_VALIDATOR_DATA: &str = "Invalid validator data has been provided";
pub const ISSUE_COST_NOT_COVERED: &str = "Native token issue cost is not covered";
pub const EGLD_TOKEN_IDENTIFIER_EXPECTED: &str =
    "The token identifier should be the EGLD token identifier";
pub const INCORRECT_OPERATION_NONCE: &str = "The operation nonce is incorrect";
pub const INVALID_FUNCTION_NOT_FOUND: &str = "invalid function (not found)";
pub const INCORRECT_DEPOSIT_AMOUNT: &str = "Incorrect deposit amount";
pub const NO_VALIDATORS_FOR_GIVEN_EPOCH: &str =
    "There are no registered validators for the given epoch";
pub const NO_VALIDATORS_FOR_PREVIOUS_EPOCH: &str =
    "There are no registered validators for the previous epoch";
