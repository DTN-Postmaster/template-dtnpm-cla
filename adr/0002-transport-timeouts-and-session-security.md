# ADR 0002: Gestion des timeouts et sécurité des sessions de transport

## Statut
Accepté

## Contexte
Les liens physiques (notamment en radiofréquence) sont sujets à des coupures brutales, à de la congestion ou à des interférences. Une opération bloquante sur un socket ou un port série peut bloquer l'ensemble du CLA et consommer indûment les ressources système.

## Décision
1. Toutes les opérations d'établissement de liaison et d'envoi de trames sont encadrées par un timeout strict (`tokio::time::timeout`).
2. Les canaux de commandes vers le pilote de transport ont une capacité bornée (backpressure) pour prévenir les dénis de service par épuisement mémoire.
3. Les sessions inactives sont purgées périodiquement.
4. L'acceptation de trames non authentifiées requiert un opt-in explicite `--allow-unauthenticated-bps-opt-in` / `DTN_HDY_ALLOW_UNAUTHENTICATED_BPS`.

## Conséquences
- Robustesse face aux blocages matériels et réseau.
- Respect strict des exigences du Cyber Resilience Act (CRA) pour les logiciels d'infrastructure.
