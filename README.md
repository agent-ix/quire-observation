# quire-observation

Private Rust implementation of qualified observation bindings and
transport-independent valuations for Quire (OB01).

The library admits records only when their selected package, producer,
binding, source/schema, subject, relationship, population/window, clock range,
and resource limits are explicit and compatible. It returns `Available`,
`Incomplete`, or `Refused`; it never treats absent evidence as Boolean false.

It accepts a selected `native-linked-package/1` descriptor and FCD 1.2.0-shaped
semantic data, but it deliberately has no compiler dependency, FCD decoder, or
telemetry adapter. OB02 owns replay/settlement and OB03 owns result reporting.

## Local checks

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test --locked
```
