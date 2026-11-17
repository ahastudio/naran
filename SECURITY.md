# Security Policy

## Supported versions

Naran has no stable release yet.
Security fixes go into the latest `main`.

## Reporting a vulnerability

Please do not open a public issue for a security problem.
Use GitHub's private reporting: open the [Security tab of the repository](https://github.com/ahastudio/naran/security/advisories/new) and choose **Report a vulnerability**.

Include what you found, how to reproduce it, and which version or commit you tried.
You will get an answer as soon as the maintainer can read the report; a fix is published together with the advisory.

## What counts

Naran reads Git repositories and runs the system `git`.
Reports about the following are welcome:

- Running a command or opening a file outside the repository a user added.
- Reading or changing files the user did not ask for, including through a crafted repository, path or ref name.
- Leaking local information from the app or from a built `Naran.app`.
