use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum VerificationError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ContractPaused = 3,
    Unauthorized = 4,
    ValidatorNotFound = 5,
    ValidatorInactive = 6,
    ValidatorAlreadyRegistered = 7,
    PlayerNotFound = 8,
    InvalidInput = 9,
    ReasonTooLong = 10,
    AlreadyConfigured = 11,
    ProgressCallFailed = 12,
    Overflow = 13,
    MilestoneNotFound = 14,
    ValidatorCapReached = 15,
    DuplicateEvidence = 16,
    MilestoneLimitExceeded = 17,
}

#[test]
fn test_approve_milestone_description_at_boundary_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    // setup: initialize contract, register validator and player
    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let client = VerificationContractClient::new(
        &env,
        &env.register_contract(None, VerificationContract {}),
    );
    // initialize and register validator (adjust args to match your actual init signature)
    client.initialize(&admin);
    client.register_validator(&admin, &validator, &String::from_str(&env, "UEFA B"));

    // Register a player in the registration contract or mock the cross-contract call
    // (adjust to match your actual test setup pattern)
    let player_id: u32 = 1;

    // 256-byte description — exactly at the limit, must succeed
    let description_256 = String::from_str(&env, &"a".repeat(256));
    let evidence = String::from_str(&env, "QmHash1234567890");
    let milestone = String::from_str(&env, "Scored 5 goals");

    let result = client.try_approve_milestone(
        &validator,
        &player_id,
        &milestone,
        &description_256,
        &evidence,
    );
    assert!(result.is_ok(), "256-byte description should succeed");
}

#[test]
fn test_approve_milestone_description_over_limit_returns_invalid_input() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let client = VerificationContractClient::new(
        &env,
        &env.register_contract(None, VerificationContract {}),
    );
    client.initialize(&admin);
    client.register_validator(&admin, &validator, &String::from_str(&env, "UEFA B"));

    let player_id: u32 = 1;

    // 257-byte description — one over the limit, must return InvalidInput
    let description_257 = String::from_str(&env, &"a".repeat(257));
    let evidence = String::from_str(&env, "QmHash1234567890");
    let milestone = String::from_str(&env, "Scored 5 goals");

    let result = client.try_approve_milestone(
        &validator,
        &player_id,
        &milestone,
        &description_257,
        &evidence,
    );
    assert_eq!(
        result,
        Err(Ok(VerificationError::InvalidInput)),
        "257-byte description should return InvalidInput"
    );
}