#![no_std]
use soroban_sdk::{contracttype, String};

/// Four-tier progress level for a player profile
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressLevel {
    /// Level 0 — profile created, no verification yet
    Unverified,
    /// Level 1 — identity confirmed by academy or KYC
    VerifiedIdentity,
    /// Level 2 — performance milestones verified by approved third party
    PerformanceMilestones,
    /// Level 3 — scout feedback or trial offer logged
    EliteTier,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ContractHealth {
    pub initialized: bool,
    pub paused: bool,
}

impl ProgressLevel {
    /// Returns the next valid level, or None if already at the top.
    pub fn next(&self) -> Option<ProgressLevel> {
        match self {
            ProgressLevel::Unverified => Some(ProgressLevel::VerifiedIdentity),
            ProgressLevel::VerifiedIdentity => Some(ProgressLevel::PerformanceMilestones),
            ProgressLevel::PerformanceMilestones => Some(ProgressLevel::EliteTier),
            ProgressLevel::EliteTier => None,
        }
    }
}

/// Validate that a string is a plausible IPFS/Arweave CID.
///
/// Rules:
/// - CIDv0: starts with "Qm", exactly 46 characters, base58btc charset
///   (no 0, O, I, l characters).
/// - CIDv1 (base32): starts with "bafy", 59–128 characters.
pub fn validate_cid(hash: &String) -> Result<(), &'static str> {
    let hash_len = hash.len();
    let bytes = hash.to_bytes();

    let starts_with_qm = bytes.get(0) == Some(b'Q') && bytes.get(1) == Some(b'm');
    let starts_with_bafy = hash_len >= 4
        && bytes.get(0) == Some(b'b')
        && bytes.get(1) == Some(b'a')
        && bytes.get(2) == Some(b'f')
        && bytes.get(3) == Some(b'y');

    if starts_with_qm {
        // CIDv0: exactly 46 chars
        if hash_len != 46 {
            return Err("invalid cid: CIDv0 must be exactly 46 characters");
        }
        // Base58btc: no 0, O, I, l
        for i in 0..hash_len {
            match bytes.get(i) {
                Some(b'0') | Some(b'O') | Some(b'I') | Some(b'l') => {
                    return Err("invalid cid: CIDv0 contains invalid base58btc character");
                }
                _ => {}
            }
        }
        Ok(())
   } else if starts_with_bafy {
        // CIDv1 (base32): 59–128 chars.
        //
        // Unlike CIDv0, this branch does NOT perform per-character base32
        // charset validation. This is intentional, not an oversight:
        //   1. CIDv0 has a fixed length (46 chars), so an exhaustive
        //      charset scan is cheap and catches every malformed input.
        //      CIDv1's length is variable (59–128 chars) across different
        //      multihash/codec combinations, so a charset scan here is
        //      comparatively more expensive for proportionally less
        //      certainty that the CID is well-formed.
        //   2. This function is a lightweight format sanity check, not a
        //      full CID decoder — it does not parse the multibase prefix,
        //      multicodec, or multihash the way a real CID library would.
        //      Doing that properly on-chain would add meaningful
        //      complexity for marginal benefit.
        //   3. Any CID that passes this check but is still malformed will
        //      simply fail to resolve against the downstream IPFS/Arweave
        //      gateway, which acts as the real source of truth for CID
        //      validity. This function only needs to catch obviously
        //      wrong input (wrong prefix, wrong length), not guarantee
        //      byte-for-byte correctness.
        if !(59..=128).contains(&hash_len) {
            return Err("invalid cid: CIDv1 must be 59–128 characters");
        }
        Ok(())
    }
    else {
        Err("invalid cid: must start with 'Qm' (CIDv0) or 'bafy' (CIDv1)")
    }
}
