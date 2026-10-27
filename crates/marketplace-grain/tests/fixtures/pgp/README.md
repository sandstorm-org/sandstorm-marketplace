Public author identity proof and public key extracted from Sandstorm package
`97291f5132208e8593a791ce6c8b63ae`, downloaded from the public app index.
No private key or application files are included.

App ID: `zxent02jw0z9ckkagz1uscdxesaf1f1wgzdunx2wrcj0jt9uf9rh`
SDK-verified fingerprint: `7116FC1150C0C96EF251B9A89324AC3921F3DE70`

The proof was emitted by Keybase as an ASCII-armored PGP MESSAGE. It reproduces
the binary-only parser's “unknown packet header version 101101” error.
It also uses a two-byte first partial-body chunk, requiring the PGP library's
legacy format compatibility feature (GnuPG accepts this encoding).

`mediawiki-public-key.pgp` and `mediawiki-proof.pgp` are the unmodified public
keyring and author proof extracted from MediaWiki SPK
`16bcca05d59606b908681c07b202752a`:
https://app-index.sandstorm.io/packages/16bcca05d59606b908681c07b202752a

App ID: `p04q48xd2kgkystkrda5n77kdns6u569wnactgh904tf7s3vev70`
SDK-verified fingerprint: `C0B2C432638DAEB0D70715C7F654B8C7D4CA3CB8`

The keyring includes a self-certified user ID plus nine third-party
certifications. Passing all of these to rPGP's self-signature verifier fails
with an issuer mismatch. The fixtures contain no private keys or application
code. Sandstorm's `spk verify` accepts the original package.

`desert-atlas-public-key.pgp` and `desert-atlas-proof.pgp` are the unmodified
public keyring and BZip2-compressed author proof extracted from Desert Atlas:
https://app-index.sandstorm.io/packages/fbc68a54911fbf5391c4ebd27ef8090a

App ID: `e5eaqnrqfrhgax1awtgw9uqayg42kcen2gkpynjs3j5mww7w3rp0`
SDK-verified fingerprint: `D05323BE2C75094F7BA017A701B0AEB646A0DF80`

These reproduce “unsupported compression algorithm BZip2” when the PGP
library's optional BZip2 support is disabled. No private keys or application
files are included.

`cleartext-public-key.pgp` and `cleartext-proof.asc` were generated locally with
GnuPG using a temporary Ed25519 signing key (whose private material was discarded).
The proof binds the same app ID as the Keybase fixture above, solely for testing.
Fingerprint: `3C4B95AB73C5C2C78569EE1BFC3958F76B94294B`.
They reproduce the packet-message parser's unsupported `CleartextMessage` error
and exercise cleartext signature verification independently of rPGP's serializer.
