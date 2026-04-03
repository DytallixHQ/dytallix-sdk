Security Policy

Reporting a Vulnerability
Dytallix is a post-quantum cryptography native blockchain. Security is the entire point. If you find a vulnerability we want to know about it immediately.
Do not open a public GitHub issue for security vulnerabilities.

How to Report
Report security vulnerabilities by sending a message directly to the maintainer on Discord:
https://discord.gg/eyVvu5kmPG
Open a direct message and include:

A description of the vulnerability
The component affected (dytallix-core, dytallix-sdk, dytallix-cli, or the chain itself)
Steps to reproduce if applicable
Your assessment of severity and exploitability
Whether you want public credit when the issue is disclosed

You will receive a response within 24 hours.

What Happens Next

We confirm receipt and begin investigation immediately
We assess severity and determine whether a fix is required before or after public disclosure
We develop and test a fix
We issue a patched release
We publish a security advisory with full details and credit to the reporter

Scope
The following are in scope for security reports:

dytallix-core: ML-DSA-65 keypair generation, Bech32m address derivation and validation, signature verification, BLAKE3 hashing
dytallix-sdk: transaction construction and signing, faucet client, keystore implementation, network client
dytallix-cli: any command that could expose private key material, bypass address validation, or allow unauthorized transactions
Chain level: consensus logic, networking layer, execution environment, fee market

Out of Scope

Issues in third-party dependencies should be reported to the dependency maintainer directly
Theoretical attacks without a practical exploit path
Issues that require physical access to the device running the node

Cryptographic Implementation Notes
We maintain an Open Problems section in the Technical Whitepaper and organization README that documents known limitations in the cryptographic implementation. If you believe one of these open problems is more serious than characterized, or if you find an issue not listed there, please report it.
Known open problems: https://github.com/DytallixHQ

Preferred Languages
English.

Disclosure Policy
We follow responsible disclosure. We ask for a reasonable window to develop and release a fix before public disclosure. We will work with you on timing and will credit you publicly unless you request otherwise.