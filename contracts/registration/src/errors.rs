use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ScoutChainError {
    /// `initialize` called more than once.
    AlreadyInitialized = 1,
    /// Operation before `initialize`.
    NotInitialized = 2,
    /// Invalid `player_id`.
    PlayerNotFound = 3,
    /// Unregistered account approving milestone.
    ValidatorNotAuthorized = 4,
    /// Skipping or reversing a level.
    InvalidProgressTransition = 5,
    /// Scout has no subscription.
    ScoutNotSubscribed = 6,
    /// Underpaying contact fee.
    InsufficientFee = 7,
    /// Wallet already has a profile for this role.
    AlreadyRegistered = 8,
    /// Circuit breaker is active.
    ContractPaused = 9,
    /// Wrong account for a privileged operation.
    Unauthorized = 10,
    /// Counter or fee arithmetic overflowed.
    Overflow = 11,
    /// Invalid `scout_id`.
    ScoutNotFound = 12,
    /// Field too long, bad hash count, or empty value.
    InvalidInput = 13,
}
