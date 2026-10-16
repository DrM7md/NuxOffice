# Security policy

## Reporting a vulnerability

Please do **not** open a public issue for security problems.

Use GitHub's private vulnerability reporting on this repository ("Security" tab, "Report a vulnerability"). If that is unavailable, contact a maintainer listed in `.github/CODEOWNERS` directly through GitHub.

Include:

- A description of the issue and its impact.
- Steps or a sample file to reproduce it. Do not attach real customer data.
- The commit or version affected.

We will acknowledge reports as quickly as we can and keep you informed while we work on a fix.

## Supported versions

NuxOffice is in Phase 0 and has no released versions yet. Security fixes land on `main`.

## Security model (summary)

The full model is described in blueprint sections 6, 7 and 12. Key commitments:

- **Opening a file never executes code.** Macros and scripts are disabled by default and require explicit, informed user consent.
- **Untrusted by default.** Workbooks, VBA modules, scripts and plugins are treated as untrusted input. Parsers must be robust against malformed and malicious files.
- **Sandboxing.** Script and macro execution is designed to run in a restricted process with capability-based permissions, time and memory limits, cancellation and logging, and no unrestricted filesystem or network access.
- **Supply chain.** Dependencies are audited for license and advisories in CI (`cargo deny`). Release artifacts will be signed where feasible.
