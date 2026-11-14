# Guide de Contribution - template-dtnpm-cla

Merci de contribuer au projet `template-dtnpm-cla`.

## Prérequis
- Rust 1.85 ou supérieur (édition 2024)
- `cargo-audit`, `cargo-deny`, `gitleaks`

## Installation des hooks git
Activez immédiatement le hook de validation pré-commit :
```bash
./scripts/install-hooks.sh
```

## Cycle de Développement
1. Implémentez ou ajustez le pilote de transport.
2. Lancez les vérifications locales :
   ```bash
   ./scripts/check-security.sh
   cargo test --workspace --all-targets
   ```
3. Respectez les conventions [Conventional Commits](https://www.conventionalcommits.org/).
