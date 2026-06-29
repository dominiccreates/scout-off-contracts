# Deployment Guide

## Prerequisites

- Rust + `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Stellar CLI: https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli
- A funded Stellar keypair for deployment

## Step-by-step

### 1. Configure environment

```bash
cp .env.example .env
# Fill in DEPLOYER_SECRET, ADMIN_ADDRESS, XLM_TOKEN_ADDRESS
```

### 2. Deploy all contracts

```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh testnet
# Contract IDs written to .env.contracts
```

### 3. Initialize and wire contracts

```bash
chmod +x scripts/initialize.sh
./scripts/initialize.sh testnet
# Sets admin, fee config, and wires all cross-contract links:
# - Verification → Progress: verification.set_progress_contract
# - Registration ← Progress: registration.set_progress_contract
# - Progress → Verification: progress.set_verification_contract
# - Progress → Registration: progress.set_registration_contract
# - Scout Access → Progress: scout_access.set_progress_contract
```

### 4. Generate TypeScript bindings

```bash
chmod +x scripts/generate-bindings.sh
./scripts/generate-bindings.sh testnet
# Bindings written to bindings/{contract}/
```

### 5. Seed testnet with demo data (optional)

```bash
chmod +x testnet/seed.sh
./testnet/seed.sh
```

### 6. Run the database migration

Copy `migrations/001_initial_schema.sql` to your backend repo and run it against PostgreSQL:

```bash
psql $DATABASE_URL -f migrations/001_initial_schema.sql
```

## Mainnet checklist

- [ ] Audit all four contracts
- [ ] Replace testnet XLM token address with mainnet address in `.env`
- [ ] Set `STELLAR_NETWORK=mainnet` and update RPC/Horizon URLs
- [ ] Run `./scripts/deploy.sh mainnet`
- [ ] Run `./scripts/initialize.sh mainnet`
- [ ] Verify all contract IDs in `.env.contracts`
- [ ] Regenerate bindings: `./scripts/generate-bindings.sh mainnet`

## Upgrading a Deployed Contract

All four contracts expose an `upgrade(new_wasm_hash)` function (admin auth required). The admin address is stored in **persistent** storage so it survives the WASM swap. Upgrading replaces only the executable WASM — **the contract ID stays the same**, so all existing clients, integrations, and indexed data continue to work without any address change.

Instance storage (Initialized, Paused, counters, fee config, contract links) is **not** automatically wiped during an upgrade, but values must be re-verified after each WASM swap in case the new code changes the storage layout or if instance TTL has drifted close to expiry.

### Scripted upgrade (recommended)

`scripts/upgrade.sh` automates the five-step procedure below, including the keypair guard, instance-state snapshot, WASM installation, the `upgrade()` call, health check, and a per-contract post-upgrade checklist.

```bash
# Build first
cargo build --target wasm32v1-none --release

# Then upgrade a single contract
./scripts/upgrade.sh testnet scout_access \
  target/wasm32v1-none/release/scoutchain_scout_access.wasm

# Other contract names: registration | verification | progress
```

The script prints a post-upgrade checklist specific to the contract being upgraded (re-wiring links, restoring fee config, regenerating bindings).

### Manual upgrade procedure

**Step 1 — Snapshot current on-chain state** (before upgrading)

```bash
# scout_access: save fee config
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  --network testnet -- get_fee_config

# All contracts: note current version
stellar contract invoke --id $CONTRACT_ID --network testnet -- version
```

**Step 2 — Build and install the new WASM**

```bash
cargo build --target wasm32v1-none --release

stellar contract install \
  --source $DEPLOYER_SECRET \
  --network testnet \
  --wasm target/wasm32v1-none/release/<contract_name>.wasm
# Prints the new wasm hash → NEW_WASM_HASH
```

**Step 3 — Call `upgrade`** (must be signed by the admin address)

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $DEPLOYER_SECRET \
  --network testnet \
  -- upgrade \
  --new_wasm_hash <NEW_WASM_HASH>
```

**Step 4 — Verify the contract is healthy**

```bash
stellar contract invoke --id $CONTRACT_ID --network testnet -- health
stellar contract invoke --id $CONTRACT_ID --network testnet -- version
```

**Step 5 — Re-apply instance state** (if needed)

For `scout_access`, restore fee config and progress contract link:

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- update_fee_config --fee_config '<saved JSON>'

stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- set_progress_contract --addr $PROGRESS_CONTRACT_ID
```

For `verification`, re-wire the progress contract link:

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- set_progress_contract \
  --progress_contract $PROGRESS_CONTRACT_ID
```

For `progress`, re-wire both cross-contract links:

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- set_verification_contract --addr $VERIFICATION_CONTRACT_ID

stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- set_registration_contract --addr $REGISTRATION_CONTRACT_ID
```

**Step 6 — Regenerate TypeScript bindings** (if the ABI changed)

```bash
./scripts/generate-bindings.sh testnet
```

### Address migration (new contract ID)

If a bug cannot be fixed via `upgrade()` (e.g. the storage layout must change in a way that requires a fresh deploy), you must migrate to a new contract address. This is a breaking change — all clients and the off-chain indexer must be updated.

Migration procedure:

1. Deploy the new contract: `./scripts/deploy.sh testnet` (or deploy just the affected contract manually).
2. Initialize the new contract: `./scripts/initialize.sh testnet`.
3. Pause the old contract so no new state is written: `stellar contract invoke --id $OLD_ID -- pause_contract`.
4. Replay any off-chain events against the new contract to seed initial state (use the backend indexer's event log).
5. Update `.env.contracts` with the new contract ID.
6. Regenerate TypeScript bindings: `./scripts/generate-bindings.sh testnet`.
7. Deploy the updated backend and frontend with the new contract ID.
8. Announce the migration in release notes with the old and new contract IDs.

### What survives an upgrade

| Data | Storage | Survives upgrade? |
|------|---------|-------------------|
| Admin address | Persistent | ✅ Yes |
| Player / scout profiles | Persistent | ✅ Yes |
| Validator registry | Persistent | ✅ Yes |
| Milestone / subscription records | Persistent | ✅ Yes |
| Contact records and scout indexes | Persistent | ✅ Yes |
| Initialized flag | Instance | ⚠️ Must re-verify |
| Paused flag | Instance | ⚠️ Must re-verify |
| Fee config (scout_access) | Instance | ⚠️ Must re-verify / re-set |
| XLM token address (scout_access) | Instance | ⚠️ Must re-verify |
| Progress contract link (all) | Instance | ⚠️ Must re-wire |

> **Note:** On Stellar, instance storage is **not** automatically wiped during an `upgrade()` call — only the contract code (WASM) is replaced. The table above reflects the risk if the new WASM changes the storage layout or if instance TTL expires before the upgrade completes. Always re-verify instance state after an upgrade using `scripts/upgrade.sh` or the manual steps above.

## Common Mistakes

**Milestones approved but player levels don't advance**
You skipped the cross-contract wiring step. `approve_milestone` calls `advance_level` on the progress contract, but only if all links have been set. Fix it by running:

```bash
./scripts/initialize.sh testnet
```

Or manually:

```bash
# 1. Verification → Progress link
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- set_progress_contract \
  --progress_contract $PROGRESS_CONTRACT_ID

# 2. Registration ← Progress link
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- set_progress_contract \
  --addr $PROGRESS_CONTRACT_ID

# 3. Progress → Verification link
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- set_verification_contract \
  --addr $VERIFICATION_CONTRACT_ID

# 4. Progress → Registration link
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- set_registration_contract \
  --addr $REGISTRATION_CONTRACT_ID

# 5. Scout Access → Progress link
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- set_progress_contract \
  --addr $PROGRESS_CONTRACT_ID
```

This must be done once after every fresh deployment.
