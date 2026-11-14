# Politique de Sécurité - template-dtnpm-cla

Le projet `template-dtnpm-cla` applique les exigences du **Cyber Resilience Act (CRA)** et les spécifications DTN (RFC 9171, RFC 9172).

## Versions supportées

| Version | Supportée |
| ------- | --------- |
| 0.1.x   | ✅ Oui    |

## Principes de Sécurité
- **Authentification & BPSec (RFC 9172)** : Mode strict par défaut. Le CLA opère en lien de confiance avec le BPA Hardy.
- **Gestion des Secrets** : Les jetons Bearer doivent impérativement être passés par fichier via `--token-file` ou `DTN_HDY_TOKEN_FILE` afin d'éviter toute exposition dans la table des processus système (`/proc/<pid>/cmdline`).
- **Protection Transport & DoS** : Encadrement systématique de toutes les opérations d'entrée/sortie par des délais d'attente (timeouts) et limitation stricte de la taille maximale des trames (`max_frame_size`).
- **Absence de panique** : Code de production garanti sans `unwrap()` ni `panic!`.

## Signalement d'une Vulnérabilité

Pour signaler une vulnérabilité de sécurité, merci de contacter l'équipe de sécurité à :
`security@dtn-postmaster.org` ou d'ouvrir un avis de sécurité privé sur GitHub.
