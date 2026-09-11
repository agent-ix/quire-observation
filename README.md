# quire-observation

Private Rust implementation of qualified observation bindings and
transport-independent valuations for Quire (OB01).

The library admits records only when their selected package, producer,
binding, source/schema, subject, relationship, population/window, clock range,
and resource limits are explicit and compatible. It returns `Available`,
`Incomplete`, or `Refused`; it never treats absent evidence as Boolean false.
It also provides a caller-selected, bounded replay kernel and typed consumer
handoff that preserve non-success dispositions and immutable dependencies.

It accepts a selected `native-linked-package/1` descriptor and FCD 1.2.0-shaped
semantic data, but it deliberately has no compiler dependency, FCD decoder, or
telemetry adapter. It does not claim production monitor execution, protocol
conformance interpretation, or evidence-store ownership.

## Local checks

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test --locked
```
