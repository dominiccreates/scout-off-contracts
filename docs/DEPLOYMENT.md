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
