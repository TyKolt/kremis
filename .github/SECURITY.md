# Security Policy

## Supported Versions

Only the latest release receives security fixes. Older versions are not backported.

## Scope

**In scope** — good-faith testing of:
- CLI (`kremis` binary) and its subcommands
- HTTP API (all endpoints, auth, rate limiting)
- MCP server (`kremis-mcp`)
- Storage backends (`redb`, `file`, in-memory)

**Out of scope:**
- Denial-of-service via resource exhaustion or malformed input
- Vulnerabilities in third-party dependencies (report those upstream; we track them via `cargo audit`)
- Issues already reported or in a public advisory

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Use [GitHub Private Security Advisories](https://github.com/TyKolt/kremis/security/advisories/new) to report a vulnerability confidentially.

### What to include

- Description of the vulnerability and its potential impact
- Steps to reproduce (minimal reproduction case preferred)
- Affected version (`kremis --version` output)
- Any suggested fix or mitigation, if available

### Response timeline

| Milestone | Target |
|-----------|--------|
| Acknowledge receipt | 48 hours |
| Initial assessment | 7 days |
| Fix or mitigation | 14 days |
| Public disclosure | 30 days after fix is released, or as jointly agreed |

If a fix requires more than 14 days, we will communicate the delay privately and agree on a new timeline before the original deadline.

Once a fix is released, a security advisory will be published on GitHub and submitted to the [RustSec Advisory Database](https://github.com/rustsec/advisory-db) so that `cargo audit` users are automatically notified.

## Safe Harbor

We consider security research conducted in good faith to be authorized under this policy. We will not pursue civil action or initiate a complaint to law enforcement for accidental, good-faith violations of this policy.

To qualify as good-faith research:
- Only test against your own instances or with explicit permission
- Avoid accessing, modifying, or deleting data that is not yours
- Do not perform DoS attacks or degrade service availability
- Report findings promptly and do not exploit vulnerabilities beyond what is necessary to demonstrate the issue

This safe harbor applies only to Kremis itself. If your research involves third-party systems or infrastructure, those parties determine their own legal stance independently of this policy.
