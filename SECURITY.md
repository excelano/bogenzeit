# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately, using the **Report a vulnerability** button on the Security tab of this repository. If that button is not there, or you would rather not go through GitHub, email david.anderson@excelano.com instead. I aim to respond within seven days.

Please do not open public issues for security problems.

## Supported versions

The latest release receives security fixes. Older versions are not supported. A fix ships in a new release; update the way you installed.

## What bz can access

bz makes no network request, opens no socket and links no networking code. It reads its arguments, the system clock and its config file, and runs no other program.

## What bz stores

Named places, in `$XDG_CONFIG_HOME/bz/config` or, without that variable, `~/.config/bz/config`. `bz place` writes the file, through a temporary file beside it that replaces it in one step, and `bz place --forget` removes a place from it. Nothing else is written.
