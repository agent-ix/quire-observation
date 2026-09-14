# quire-observation

Private Rust implementation of qualified observation bindings and
transport-independent valuations for Quire (OB01).

The library admits records only when their selected package, producer,
binding, source/schema, subject, relationship, population/window, clock range,
and resource limits are explicit and compatible. It returns `Available`,
`Incomplete`, or `Refused`; it never treats absent evidence as Boolean false.
It also publishes nine canonical bounded owner contracts for observation,
population, position, clock, capture, progress, closure, completeness, and
result availability. Consumers receive only strict-read validated views.

It accepts a selected `native-linked-package/1` descriptor and FCD 1.2.0-shaped
semantic data, but it deliberately has no compiler dependency, FCD decoder, or
telemetry adapter, temporal evaluator, or protocol-result mapper. It does not
claim production monitor execution, protocol conformance interpretation, or
evidence-store ownership.

## Owner contracts

| Module | Contract | Schema SHA-256 | Public reader |
| --- | --- | --- | --- |
| `authority::observation` | `quire.observation.record/v1` | `2922c6ad6bbca53f0800b7bd5159781632aa6ebd1e696567ae5639ec18dea3a3` | `observation::read` |
| `authority::population` | `quire.observation.population/v1` | `493b4c10701356007d055d351171ee1897de2b7226da03161fa86a9e52e524c2` | `population::read` |
| `authority::position` | `quire.observation.position-ledger/v1` | `aac2fd7fc1b129e24afec397900647f3fe8977341b9907662a3852ac5430211a` | `position::read` |
| `authority::clock` | `quire.observation.clock-binding/v1` | `c9a7b154a2c2d775ba71ba001842e6f0595e123c07b23a2bb37215c1f05de38d` | `clock::read` |
| `authority::capture` | `quire.observation.capture-environment/v1` | `6d9260d3c4de65b5c3304debdb9baa7816aaea55949839924182d1db4c24c516` | `capture::read` |
| `authority::progress` | `quire.observation.progress-assertion/v1` | `91223fb982b68d4fb7c66b3737370429bd4841321f5b561ebc6c1e9691bd7f1c` | `progress::read` |
| `authority::closure` | `quire.observation.closure-assertion/v1` | `8638fb9f4dd5a26d44ae9975388fc734670992914422e23c27eaae82149245f4` | `closure::read` |
| `authority::completeness` | `quire.observation.completeness-assertion/v1` | `b7c3c48d5f907cfb5bd7b4450e009b6c95f7d558095ece8b81d2ae1d5821bcf0` | `completeness::read` |
| `authority::availability` | `quire.observation.result-availability/v1` | `2e6c00d6dc94a00b0859b346dc12a14c7142735a565e35d15b85895d311e8416` | `availability::read` |

Population admission additionally strict-reads
`quire.observation.explicit-members/v1` through
`population::read_membership`; its schema digest is
`408c9d2908c3660d657cea574f52d8531e9ab749ec78c8d058fe5f163df23cce`.

## Local checks

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test --locked
```
