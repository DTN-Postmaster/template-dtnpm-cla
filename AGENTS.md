# Guidelines for Autonomous Agents & AI Assistants — `template-dtnpm-cla`

This reference applies strictly to any AI agent intervention on the `template-dtnpm-cla` repository.

## Documentation Language
English must be used as the language for all documentation, unless the file is suffixed with `.fr.md` or another language is explicitly requested in the prompt.

## 1. Architecture Rules
- Every CLA adapter must implement the `hardy_bpa::cla::Cla` trait.
- Low-level transport logic must be isolated in an asynchronous `Driver` connected via bounded channels (`mpsc`).
- Neighbor resolution is performed via `peer_resolver`, converting a `NodeId` into a `ClaAddress`.

## 2. Mandatory Security Standards
- **Zero `unwrap()` and zero `panic!`** in production code (`src/lib.rs`, `src/cla.rs`, `src/driver.rs`, `src/config.rs`, `src/error.rs`, `src/main.rs`). The `-D clippy::unwrap_used` and `-D clippy::panic` lints are enabled.
- **Secret leakage protection**: Never require `--token` on the command line; enforce `--token-file` / `DTN_HDY_TOKEN_FILE`.
- **Timeouts and rate limiting**: All I/O and network wait operations must be time-bounded.
- **Strict BPSec by default**: Never relax `VerifyPolicy::Strict` unless explicitly requested with the `--allow-unauthenticated-bps-opt-in` option.

## 3. Validation Procedure
Before committing any change:
```bash
./scripts/check-security.sh --strict
cargo test --workspace --all-targets
```
Any compiler warning (`cargo clippy`) or audit failure (`cargo-deny`, `gitleaks`) is blocking.
