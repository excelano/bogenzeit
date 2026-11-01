# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub Security Advisories at https://github.com/excelano/bogenzeit/security/advisories/new. If you would rather not use GitHub, email david.anderson@excelano.com instead. I aim to respond within seven days.

Please do not open public issues for security problems.

## Supported versions

The latest release receives fixes. A security fix ships through `main` in a new release on GitHub and through the Excelano apt repository; there are no maintained release branches.

## Security model

**bz has no network surface.** It makes no network request, opens no socket and links no networking code; its only inputs are its arguments and the system clock.
