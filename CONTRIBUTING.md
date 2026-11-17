# Contributing Guide — `template-dtnpm-cla`

Thank you for contributing to the `template-dtnpm-cla` project.

## Prerequisites
- Rust 1.85 or later (2024 edition)
- `cargo-audit`, `cargo-deny`, `gitleaks`

## Installing Git Hooks
Immediately enable the pre-commit validation hook:
```bash
./scripts/install-hooks.sh
```

## Development Cycle
1. Implement or adjust the transport driver.
2. Run local checks:
   ```bash
   ./scripts/check-security.sh
   cargo test --workspace --all-targets
   ```
3. Follow [Conventional Commits](https://www.conventionalcommits.org/) conventions.
