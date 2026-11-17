# Security Policy — `template-dtnpm-cla`

The `template-dtnpm-cla` project enforces **Cyber Resilience Act (CRA)** requirements and DTN specifications (RFC 9171, RFC 9172).

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅ Yes    |

## Security Principles
- **Authentication & BPSec (RFC 9172)**: Strict mode by default. The CLA operates in a trusted relationship with the Hardy BPA.
- **Secret Management**: Bearer tokens must be passed via file using `--token-file` or `DTN_HDY_TOKEN_FILE` to avoid any exposure in the system process table (`/proc/<pid>/cmdline`).
- **Transport & DoS Protection**: Systematic wrapping of all I/O operations with timeouts and strict enforcement of maximum frame size (`max_frame_size`).
- **No Panics**: Production code guaranteed free of `unwrap()` and `panic!`.

## Reporting a Vulnerability

To report a security vulnerability, please contact the security team at:
`security@dtn-postmaster.org` or open a private Security Advisory on GitHub.
