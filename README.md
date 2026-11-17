# Template Convergence Layer Adapter (`template-dtnpm-cla`)

Reference template to quickly create a new Convergence Layer Adapter (CLA) for DTN Postmaster and the Hardy BPA (`hardy_bpa::cla::Cla`).

## Included Features
- Full implementation of the [`hardy_bpa::cla::Cla`] trait.
- Background asynchronous driver (`Driver`) with robust session management and bounded channels.
- Automatic gRPC / UDS registration with the BPA daemon (`RemoteBpa`).
- Dynamic neighbor resolution (`peer_resolver`) mapping `NodeId` to `ClaAddress`.
- Built-in security compliant with the **Cyber Resilience Act (CRA)** and DTN standards:
  - Support for `--token-file` / `DTN_HDY_TOKEN_FILE` avoiding token exposure in the process table.
  - Exception option `--allow-unauthenticated-bps-opt-in` for testbench use.
  - Guaranteed absence of `unwrap()` and `panic!` in production code.
  - Automated audit and compliance checks (`./scripts/check-security.sh`).

## How to Instantiate This Template

1. Clone or duplicate this repository under your adapter's name (e.g., `dtnpm-cla-ax25`).
2. Update `name = "dtnpm-cla-ax25"` in `Cargo.toml`.
3. Rename `TemplateCla` in `src/cla.rs` and adapt the transmission logic in `src/driver.rs`.
4. Adjust the configuration in `src/config.rs` and the example in `examples/`.
5. Validate with `./scripts/check-security.sh` and run tests: `cargo test`.
