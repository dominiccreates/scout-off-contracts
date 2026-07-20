# Contributing

## Prerequisites

Ensure the following tools are installed at the specified minimum versions before attempting to build or test the contracts. Mismatched versions are the most common cause of opaque WASM build failures.

| Tool | Minimum version | Install / notes |
|------|----------------|-----------------|
| **Rust** (via rustup) | stable (1.78+) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **WASM build target** | `wasm32v1-none` | `rustup target add wasm32v1-none` |
| **cargo** | ships with Rust stable | Verify: `cargo --version` |
| **clippy** | ships with Rust stable | `rustup component add clippy` |
| **rustfmt** | ships with Rust stable | `rustup component add rustfmt` |
| **stellar-cli** | **21.6.0** (pinned) | See install note below |
| **Node.js** | 20 LTS | Required only for TypeScript bindings generation — `./scripts/generate-bindings.sh` |
| **npm** | 10+ (ships with Node 20) | Required only for building/testing bindings packages |

> CI uses `dtolnay/rust-toolchain@stable` and installs `stellar-cli` at the pinned version listed above. If a local build diverges from CI, update your Rust toolchain (`rustup update stable`) and reinstall stellar-cli at the pinned version.

### Installing the pinned stellar-cli version

`scripts/generate-bindings.sh` enforces the required `stellar-cli` version and will fail with a
clear error if the wrong version is detected. Install the exact version with:

```bash
curl -sSL https://raw.githubusercontent.com/stellar/stellar-cli/v21.6.0/install.sh | bash
```

Then verify: `stellar --version` should print `stellar 21.6.0`.

The `wasm32v1-none` target (not the older `wasm32-unknown-unknown`) is required for building Soroban contracts with `soroban-sdk 25.x`. Using the wrong target produces an ABI-incompatible WASM binary.

## Setup

```bash
rustup target add wasm32-unknown-unknown
rustup component add clippy rustfmt
cp .env.example .env
```

## Before opening a PR

```bash
cargo test --workspace          # all tests must pass
cargo clippy --workspace        # zero warnings
cargo fmt --all -- --check      # formatting must be clean
```

### CI jobs: what you can check locally

`docs/CONTRIBUTING.md` only lists three commands above, but CI actually runs
five jobs. The table below covers all five, and whether you can catch their
failures before you push.

| CI job | What it checks | Locally reproducible? | Command(s) |
|---|---|---|---|
| `check-todos` | Blocks `TODO`/`FIXME`/`HACK`/`XXX` markers in `contracts/**/*.rs` | ✅ Yes | `grep -rIn -E '\b(TODO\|FIXME\|HACK\|XXX)\b' contracts/ --include='*.rs'` |
| `test` | Full workspace test suite, an extra verbose run of the `progress` contract's tests, and a release WASM build | ✅ Yes (already listed above) | `cargo test --workspace` (optionally also `cargo build --workspace --target wasm32v1-none --release`) |
| `lint` | Clippy (zero warnings), `rustfmt --check`, `shellcheck` on every script in `scripts/` and `testnet/seed.sh`, a docs-completeness check, and bindings `package.json` template validation | ✅ Yes | `cargo clippy --workspace -- -D warnings` · `cargo fmt --all -- --check` · `shellcheck scripts/*.sh testnet/seed.sh` (requires `shellcheck` installed) · `bash scripts/check-docs.sh` · `bash scripts/check-bindings.sh` |
| `bindings-smoke-test` | Boots a Dockerized local Soroban sandbox (`stellar/quickstart:testing`), deploys all four contracts to it, then generates and builds the TypeScript bindings packages against that live deployment | ⚠️ CI-only in practice | Technically reproducible if you have Docker — see below — but the setup cost (Docker, a *second* pinned Stellar CLI version, funding a sandbox identity) makes it impractical to run on every push. Most contributors should treat this as CI-only feedback. |
| `abi-export` | Builds release WASM and exports each contract's ABI as JSON via `stellar contract info interface`, then validates the JSON parses | ✅ Partially | `cargo build --workspace --target wasm32v1-none --release`, then for each contract: `stellar contract info interface --wasm target/wasm32v1-none/release/scoutchain_<contract>.wasm --output json-formatted`. The artifact upload (for diffing ABIs across commits) is GitHub Actions storage with no local equivalent, but you don't need it to self-verify — just check the printed JSON for unexpected changes. |

#### Reproducing `bindings-smoke-test` locally (optional)

This job requires Docker and, importantly, a **different pinned Stellar CLI
version (25.2.0)** than the 21.6.0 pinned above for everyday development —
installing both side by side is extra friction most contributors won't want
for routine PRs. If you do want to reproduce it:

```bash
# 1. Build release WASM
cargo build --workspace --target wasm32v1-none --release

# 2. Install the CLI version this job actually uses (25.2.0, not 21.6.0)
curl -sSL https://raw.githubusercontent.com/stellar/stellar-cli/v25.2.0/install.sh | bash

# 3. Start a local Soroban sandbox
docker run -d --name stellar-local -p 8000:8000 stellar/quickstart:testing --local

# 4. Register the local network, then generate + fund a deployer identity
stellar network add local \
  --rpc-url http://localhost:8000/soroban/rpc \
  --network-passphrase "Standalone Network ; February 2017"
stellar keys generate ci-deployer --network local
stellar keys fund ci-deployer --network local

# 5. Deploy each contract (see the deploy_one() loop in
#    .github/workflows/contract-ci.yml for the exact build/optimize/deploy
#    steps), then generate and build the bindings
bash scripts/generate-bindings.sh local

# 6. Tear down
docker stop stellar-local && docker rm stellar-local
```

Given the setup cost, it's reasonable to skip this locally and rely on CI's
`bindings-smoke-test` run for this specific feedback.

## Contract change checklist

- [ ] New functions have unit tests covering the happy path and at least one error case
- [ ] Any new `DataKey` variant is documented with a comment
- [ ] Cross-contract calls are documented with a comment explaining the atomicity guarantee
- [ ] `ai.md` is updated if shared types, events, or env vars changed
- [ ] `docs/CONTRACT_REFERENCE.md` is updated with new functions

### Error variant ordering

Every contract's `#[contracterror]` enum (`errors.rs`) is append-only.
**New error variants must be added at the end of the enum, never inserted
between existing variants and never renumbered.** This matches the
`docs/VERSIONING.md` policy: renumbering or removing an existing
`#[contracterror]` variant is a MAJOR breaking change because external
consumers, on-chain event listeners, and off-chain indexers key off the
numeric code.

When adding a new variant:

- Append it after the last existing variant. Do not "fill gaps" in the
  numeric sequence — a gap (e.g. `12 → 14`) is a deliberate reservation,
  not a bug, and must be preserved.
- Group related variants together by inserting a brief section comment
  above the group (e.g. `// ── Rate limiting ──`). Grouping is purely
  cosmetic for readers; numeric contiguity within a group is **not**
  required and **not** guaranteed by this convention.
- If the new variant belongs to an existing group, place it at the end
  of that group rather than at the end of the enum, so the grouping
  remains readable. This does not violate append-only because the
  variant's numeric code is the next free value after the current
  maximum — readers can still scan the file top-to-bottom to find it.
- Do not reuse a numeric code that has been removed in a prior version,
  even if the variant is long-since deprecated. On-chain history may
  still reference it.

Rationale and the full set of breaking-change rules live in
[`docs/VERSIONING.md`](VERSIONING.md).

## Validator authorization changes

Changes to validator registration, revocation, or milestone approval logic require explicit
review from a second team member before merge — these are the trust anchors of the platform.

## Glossary

Unfamiliar with terms like *validator*, *milestone*, *subscription tier*, or *CID*?
See [docs/GLOSSARY.md](GLOSSARY.md) for authoritative definitions of all domain-specific terms.