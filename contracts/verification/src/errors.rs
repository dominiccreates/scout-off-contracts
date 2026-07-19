use soroban_sdk::contracterror;

/// Append-only: do not renumber existing variants. See docs/CONTRIBUTING.md.
#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum VerificationError {
    // ── Initialization & lifecycle ──
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ContractPaused = 3,
    AlreadyConfigured = 11,

    // ── Authorization ──
    Unauthorized = 4,

    // ── Validators ──
    ValidatorNotFound = 5,
    ValidatorInactive = 6,
    ValidatorAlreadyRegistered = 7,
    ValidatorCapReached = 15,

    // ── Milestones & evidence ──
    PlayerNotFound = 8,
    MilestoneNotFound = 14,
    DuplicateEvidence = 16,
    MilestoneLimitExceeded = 17,

    // ── Input validation ──
    InvalidInput = 9,
    ReasonTooLong = 10,

    // ── Cross-contract & arithmetic ──
    ProgressCallFailed = 12,
    Overflow = 13,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    const VALID_CID_V0: &str = "QmPK1s3pNYLi9ERiq3BDxKa4XosgWwFRQUydHUtz4YgpqB";

    fn setup() -> (Env, crate::VerificationContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, crate::VerificationContract);
        let client = crate::VerificationContractClient::new(&env, &id);
        (env, client)
    }

    #[test]
    fn test_approve_milestone_description_at_boundary_succeeds() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        let validator = Address::generate(&env);
        client.initialize(&admin);
        client.register_validator(&validator, &String::from_str(&env, "UEFA B License"));

        let description_256 = String::from_str(&env, &"a".repeat(256));
        let evidence = String::from_str(&env, VALID_CID_V0);

        let result = client.try_approve_milestone(&validator, &1u64, &description_256, &evidence);
        assert!(result.is_ok(), "256-byte description should succeed");
    }

    #[test]
    fn test_approve_milestone_description_over_limit_returns_invalid_input() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        let validator = Address::generate(&env);
        client.initialize(&admin);
        client.register_validator(&validator, &String::from_str(&env, "UEFA B License"));

        let description_257 = String::from_str(&env, &"a".repeat(257));
        let evidence = String::from_str(&env, VALID_CID_V0);

        let result = client.try_approve_milestone(&validator, &1u64, &description_257, &evidence);
        assert_eq!(
            result,
            Err(Ok(VerificationError::InvalidInput)),
            "257-byte description should return InvalidInput"
        );
    }
}
