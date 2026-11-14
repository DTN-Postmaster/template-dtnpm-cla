# Directives pour Agents Autonomes & Assistants IA - template-dtnpm-cla

Ce référentiel s'applique impérativement à toute intervention d'un agent IA sur le dépôt `template-dtnpm-cla`.

## 1. Règles d'Architecture
- Tout adaptateur CLA doit implémenter le trait `hardy_bpa::cla::Cla`.
- La logique de transport bas niveau doit être isolée dans un `Driver` asynchrone connecté via des canaux bornés (`mpsc`).
- La résolution de voisins s'effectue via `peer_resolver` convertissant un `NodeId` en `ClaAddress`.

## 2. Standards de Sécurité Obligatoires
- **Zéro `unwrap()` et zéro `panic!`** dans le code de production (`src/lib.rs`, `src/cla.rs`, `src/driver.rs`, `src/config.rs`, `src/error.rs`, `src/main.rs`). Les lints `-D clippy::unwrap_used` et `-D clippy::panic` sont activés.
- **Protection contre les fuites de secrets** : Ne jamais imposer `--token` en ligne de commande ; imposer `--token-file` / `DTN_HDY_TOKEN_FILE`.
- **Timeouts et limitation de débit** : Toutes les opérations I/O et d'attente réseau doivent être bornées dans le temps.
- **BPSec Strict par défaut** : Ne jamais relâcher la politique `VerifyPolicy::Strict` sauf demande explicite avec l'option `--allow-unauthenticated-bps-opt-in`.

## 3. Procédure de Validation
Avant de valider tout commit :
```bash
./scripts/check-security.sh --strict
cargo test --workspace --all-targets
```
Tout avertissement du compilateur (`cargo clippy`) ou échec d'audit (`cargo-deny`, `gitleaks`) est bloquant.
