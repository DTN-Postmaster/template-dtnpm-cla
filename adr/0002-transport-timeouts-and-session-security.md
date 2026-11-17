# ADR 0002: Transport Timeouts and Session Security Management

## Status
Accepted

## Context
Physical links (especially over RF) are prone to abrupt drops, congestion, or interference. A blocking operation on a socket or serial port can stall the entire CLA and unduly consume system resources.

## Decision
1. All link establishment and frame transmission operations are wrapped in a strict timeout (`tokio::time::timeout`).
2. Command channels to the transport driver have bounded capacity (backpressure) to prevent denial-of-service by memory exhaustion.
3. Inactive sessions are purged periodically.
4. Accepting unauthenticated frames requires an explicit opt-in `--allow-unauthenticated-bps-opt-in` / `DTN_HDY_ALLOW_UNAUTHENTICATED_BPS`.

## Consequences
- Robustness against hardware and network stalls.
- Strict compliance with Cyber Resilience Act (CRA) requirements for infrastructure software.
