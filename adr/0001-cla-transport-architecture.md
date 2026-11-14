# ADR 0001: Architecture du Convergence Layer Adapter (CLA)

## Statut
Accepté

## Contexte
Hardy BPA route des paquets BPv7 (RFC 9171) à travers diverses couches de transport réseau et radio (TCPCLv4, Mercury, AX.25, LoRa, UDP, LTP). Le trait `hardy_bpa::cla::Cla` standardise l'interface entre le moteur de routage du BPA et le pilote de transport.

## Décision
1. Chaque adaptateur implémente le trait `hardy_bpa::cla::Cla`.
2. La gestion du transport matériel ou réseau est déportée dans un travailleur d'arrière-plan (`Driver`) via une file asynchrone bornée (`tokio::sync::mpsc`).
3. L'adaptateur déclare explicitement ses capacités à l'enregistrement via `ClaInit` (nombre de voies/lanes, taille maximale de trame).
4. La résolution de voisin `peer_resolver` convertit un EID de nœud (`NodeId`) en adresse physique locale (`ClaAddress`).

## Conséquences
- Découplage complet entre la logique BPA et les particularités de bas niveau du protocole de transport.
- Résilience aux pannes réseau grâce à la gestion asynchrone des transferts.
