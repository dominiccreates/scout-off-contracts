#!/usr/bin/env bash
# ScoutChain — cross-contract wiring diagnostic
#
# Queries every get_*_contract-style getter across all four contracts and
# prints a clear pass/fail status table for all five documented wiring links.
#
# Usage:
#   ./scripts/verify-cross-contract-wiring.sh [testnet|mainnet|local]
#
# Requires .env.contracts to exist (written by deploy.sh / initialize.sh).
# Read-only: this script makes no state changes and is safe to run against
# production at any time.
#
# Exit codes:
#   0  — all five links are wired correctly
#   1  — one or more links are missing or could not be queried
#
# See docs/DEPLOYMENT.md "Common Mistakes" and ai.md "ProgressCallFailed"
# for the full wiring procedure this script diagnoses.
set -euo pipefail

NETWORK="${1:-testnet}"

# ---------------------------------------------------------------------------
# Load contract IDs
# ---------------------------------------------------------------------------
if [[ ! -f .env.contracts ]]; then
  echo "ERROR: .env.contracts not found." >&2
  echo "       Run ./scripts/deploy.sh $NETWORK first to create it." >&2
  exit 1
fi

# shellcheck source=/dev/null
source .env.contracts

REGISTRATION_CONTRACT_ID="${REGISTRATION_CONTRACT_ID:?REGISTRATION_CONTRACT_ID not set in .env.contracts}"
VERIFICATION_CONTRACT_ID="${VERIFICATION_CONTRACT_ID:?VERIFICATION_CONTRACT_ID not set in .env.contracts}"
PROGRESS_CONTRACT_ID="${PROGRESS_CONTRACT_ID:?PROGRESS_CONTRACT_ID not set in .env.contracts}"
SCOUT_ACCESS_CONTRACT_ID="${SCOUT_ACCESS_CONTRACT_ID:?SCOUT_ACCESS_CONTRACT_ID not set in .env.contracts}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# query_link <description> <contract_id> <getter_fn> <expected_id>
#
# Calls <getter_fn> on <contract_id>, compares the returned address against
# <expected_id>, and sets LINK_FAILED=1 if they don't match.
#
# Prints one table row:
#   ✅  <description>  →  <returned_address>
#   ❌  <description>  →  NOT WIRED  (expected <expected_id>)
LINK_FAILED=0

query_link() {
  local description="$1"
  local contract_id="$2"
  local getter_fn="$3"
  local expected_id="$4"

  local result
  result=$(stellar contract invoke \
    --id "$contract_id" \
    --network "$NETWORK" \
    -- "$getter_fn" 2>/dev/null || true)

  # stellar contract invoke wraps string output in quotes; strip them.
  local returned
  returned=$(echo "$result" | tr -d '"' | tr -d '[:space:]')

  if [[ -z "$returned" ]]; then
    printf "  ❌  %-52s  →  NOT WIRED  (expected %s)\n" "$description" "$expected_id"
    LINK_FAILED=1
  elif [[ "$returned" == "$expected_id" ]]; then
    printf "  ✅  %-52s  →  %s\n" "$description" "$returned"
  else
    printf "  ❌  %-52s  →  WRONG ADDRESS: %s\n" "$description" "$returned"
    printf "      %-52s     (expected: %s)\n" "" "$expected_id"
    LINK_FAILED=1
  fi
}

# ---------------------------------------------------------------------------
# Header
# ---------------------------------------------------------------------------
echo ""
echo "ScoutChain — cross-contract wiring check (network: $NETWORK)"
echo "============================================================="
echo ""
echo "  Contract IDs loaded from .env.contracts:"
echo "    REGISTRATION  $REGISTRATION_CONTRACT_ID"
echo "    VERIFICATION  $VERIFICATION_CONTRACT_ID"
echo "    PROGRESS      $PROGRESS_CONTRACT_ID"
echo "    SCOUT_ACCESS  $SCOUT_ACCESS_CONTRACT_ID"
echo ""
echo "  Querying all five wiring links..."
echo ""

# ---------------------------------------------------------------------------
# Five wiring links documented in docs/DEPLOYMENT.md "Common Mistakes"
# and wired by initialize.sh:
#
#   1. verification  → progress   (verification.set_progress_contract)
#   2. registration  → progress   (registration.set_progress_contract)
#   3. progress      → verification (progress.set_verification_contract)
#   4. progress      → registration (progress.set_registration_contract)
#   5. scout_access  → progress   (scout_access.set_progress_contract)
# ---------------------------------------------------------------------------

query_link \
  "verification → progress  (get_progress_contract)" \
  "$VERIFICATION_CONTRACT_ID" \
  "get_progress_contract" \
  "$PROGRESS_CONTRACT_ID"

query_link \
  "registration → progress  (get_progress_contract)" \
  "$REGISTRATION_CONTRACT_ID" \
  "get_progress_contract" \
  "$PROGRESS_CONTRACT_ID"

query_link \
  "progress → verification  (get_verification_contract)" \
  "$PROGRESS_CONTRACT_ID" \
  "get_verification_contract" \
  "$VERIFICATION_CONTRACT_ID"

query_link \
  "progress → registration  (get_registration_contract)" \
  "$PROGRESS_CONTRACT_ID" \
  "get_registration_contract" \
  "$REGISTRATION_CONTRACT_ID"

query_link \
  "scout_access → progress  (get_progress_contract)" \
  "$SCOUT_ACCESS_CONTRACT_ID" \
  "get_progress_contract" \
  "$PROGRESS_CONTRACT_ID"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
if [[ "$LINK_FAILED" -ne 0 ]]; then
  echo "  RESULT: ❌  One or more wiring links are missing or incorrect."
  echo ""
  echo "  To fix, re-run initialize.sh (or apply individual wiring commands):"
  echo "    ./scripts/initialize.sh $NETWORK"
  echo ""
  echo "  See docs/DEPLOYMENT.md 'Common Mistakes' for the manual wiring commands."
  echo "  See ai.md 'ProgressCallFailed' for SDK-level recovery steps."
  echo ""
  exit 1
else
  echo "  RESULT: ✅  All five wiring links are correctly set."
  echo ""
fi
