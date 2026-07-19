use soroban_sdk::contracterror;

/// Append-only: do not renumber existing variants. See docs/CONTRIBUTING.md.
#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ScoutChainError {
    // ── Initialization & lifecycle ──
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ContractPaused = 9,

    // ── Authorization ──
    ValidatorNotAuthorized = 4,
    Unauthorized = 10,

    // ── Registration & lookup ──
    AlreadyRegistered = 8,
    PlayerNotFound = 3,
    ScoutNotFound = 12,

    // ── Business logic ──
    InvalidProgressTransition = 5,
    ScoutNotSubscribed = 6,
    InsufficientFee = 7,

    // ── Validation & arithmetic ──
    InvalidInput = 13,
    Overflow = 11,
}
