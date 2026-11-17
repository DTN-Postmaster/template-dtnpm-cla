# ADR 0001: Convergence Layer Adapter (CLA) Architecture

## Status
Accepted

## Context
Hardy BPA routes BPv7 packets (RFC 9171) across diverse network and radio transport layers (TCPCLv4, Mercury, AX.25, LoRa, UDP, LTP). The `hardy_bpa::cla::Cla` trait standardizes the interface between the BPA routing engine and the transport driver.

## Decision
1. Every adapter implements the `hardy_bpa::cla::Cla` trait.
2. Hardware or network transport management is offloaded to a background worker (`Driver`) via a bounded asynchronous queue (`tokio::sync::mpsc`).
3. The adapter explicitly declares its capabilities upon registration via `ClaInit` (number of lanes, maximum frame size).
4. Neighbor resolution `peer_resolver` converts a node EID (`NodeId`) into a local physical address (`ClaAddress`).

## Consequences
- Complete decoupling between BPA logic and low-level transport protocol specifics.
- Resilience to network failures through asynchronous transfer handling.
