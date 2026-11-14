# Template Convergence Layer Adapter (`template-dtnpm-cla`)

Modèle de référence pour créer rapidement un nouvel adaptateur de couche de convergence (CLA) pour DTN Postmaster et le BPA Hardy (`hardy_bpa::cla::Cla`).

## Fonctionnalités incluses
- Implémentation complète du trait [`hardy_bpa::cla::Cla`].
- Pilote asynchrone d'arrière-plan (`Driver`) avec gestion robuste des sessions et des canaux bornés.
- Enregistrement automatique gRPC / UDS auprès du daemon BPA (`RemoteBpa`).
- Résolution dynamique de voisins (`peer_resolver`) associant `NodeId` et `ClaAddress`.
- Sécurité intégrée conforme au **Cyber Resilience Act (CRA)** et aux standards DTN :
  - Support de `--token-file` / `DTN_HDY_TOKEN_FILE` évitant l'exposition de jetons dans la table des processus.
  - Option d'exception `--allow-unauthenticated-bps-opt-in` pour le banc de test.
  - Absence garantie de `unwrap()` et de `panic!` dans le code de production.
  - Vérifications automatisées d'audit et de conformité (`./scripts/check-security.sh`).

## Comment instancier ce template

1. Cloner ou dupliquer ce dépôt sous le nom de votre adaptateur (ex: `dtnpm-cla-ax25`).
2. Mettre à jour `name = "dtnpm-cla-ax25"` dans `Cargo.toml`.
3. Renommer `TemplateCla` dans `src/cla.rs` et adapter la logique de transmission dans `src/driver.rs`.
4. Ajuster la configuration dans `src/config.rs` et l'exemple dans `examples/`.
5. Valider avec `./scripts/check-security.sh` et exécuter les tests : `cargo test`.
