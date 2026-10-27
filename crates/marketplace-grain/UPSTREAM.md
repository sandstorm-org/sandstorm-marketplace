# Sandstorm protocol provenance

The schemas in `schema/sandstorm` come from Sandstorm commit
`03e28f425160c86a3a8a379277ad62f675ece0ae`, under the Apache 2.0 license.
Their copyright and license notices are retained. `schema/capnp` contains
Cap'n Proto support schemas with their original license notices.

The Rust grain implements the app-index submission/storage protocols described
by these schemas. It contains no copied C++ implementation. The SDK remains an
optional independent reference for interoperability tests.

`catalog.rs` carries the original public catalog's short-description overrides
for Let's Chat, MediaGoblin, FileDrop, and Paperwork, keyed by app ID. The old
indexer loaded these separately from `/var/descriptions`; they were not part of
the signed manifests. Upload validation and listing generation use the same
overrides, without changing stored verified metadata or SPK bytes.

Local schema adaptations preserve wire IDs and field ordinals:

- Rename the package `categoryInfo` annotation to `categoryTitle` to avoid a Rust
  name collision with its struct.
- Rename the `persistent` annotation to `persistentAnnotation` for the same reason.
- Omit the obsolete embedded app-index review HTML and package definition.

`build.rs` uses `capnpc` to generate Rust bindings. It patches one generated
annotation accessor to work around capnpc 0.21's handling of an enum inside a
generic supervisor struct; the enum has no generic fields. This does not change
the protocol. The `c++.capnp` file is an annotation schema imported by upstream
schemas, not C++ source or a compiled runtime dependency.
