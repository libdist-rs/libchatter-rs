# libchatter-rs (archived)

This repository is the **FC 2023 Apollo artefact** -- a Rust implementation of
the Apollo consensus protocol along with three sibling BFT protocols
(Artemis, Sync HotStuff, Opt Sync) and the support libraries they share.

**Active development has moved.** The four consensus protocols are being
re-homed in [`libdist-rs/libapollo-rs`](https://github.com/libdist-rs/libapollo-rs),
rebuilt against the cleaned-up successor libraries
[`libnet-rs`](https://github.com/libdist-rs/libnet-rs),
[`libcrypto-rs`](https://github.com/libdist-rs/libcrypto-rs),
[`libstorage-rs`](https://github.com/libdist-rs/libstorage-rs), and
[`libmempool-rs`](https://github.com/libdist-rs/libmempool-rs).

This repository is kept read-only for reproducibility of the published paper.
Please open issues / PRs against `libapollo-rs`.

## Protocols in this repo

| Crate               | Protocol              | Binary pair                                         |
| ------------------- | --------------------- | --------------------------------------------------- |
| `consensus/apollo`  | Apollo (FC 2023)      | `node-apollo`, `client-apollo`, `normal-client-apollo` |
| `consensus/artemis` | Artemis               | `node-artemis`, `client-artemis`                    |
| `consensus/synchs`  | Sync HotStuff         | `node-synchs`, `client-synchs`                      |
| `consensus/optsync` | Opt Sync              | `node-optsync`, `client-optsync`                    |

## Building and running

```sh
# Build binaries + the config generator
cargo build --release

# Generate the fixed testdata directories (n=3 and n=9 clusters, various block sizes)
make testdata

# Run a quick single-protocol test
bash scripts/apollo-release-quick-test.sh
# or: synchs-release-quick-test.sh, optsync-test.sh, artemis-test.sh, ...
```

## Baseline stress test

`stress-test/` is a Rust harness that drives all four protocols on the loopback
interface, parses throughput / latency from the client's `consensus::statistics`
output, and prints a single comparable report. This is the canonical reference
for "what does the FC artefact do on this machine". Numbers from a clean run
live in [`baseline_results.txt`](baseline_results.txt).

```sh
cargo build --release
./target/release/stress-test
```

The harness is intentionally lift-and-shift over the existing example binaries
(no changes to the protocol code) -- its job is to capture an honest baseline
before `libapollo-rs` migrates to the modern successor libraries, so that
post-migration numbers can be held against it.

## Repo layout

- `consensus/` -- protocol implementations (apollo, artemis, synchs, optsync, dummy)
- `examples/<protocol>/{node,client}/` -- runnable binaries per protocol
- `config/` -- `Node` / `Client` config structs + (de)serialization
- `crypto/` -- ED25519, SECP256K1, SHA256 (RSA stubbed)
- `net/` -- TLS-authenticated `futures_manager::TlsClient<I,O>` (used by Apollo/Artemis)
  and `tokio_manager` (used by Sync HotStuff / Opt Sync)
- `types/` -- per-protocol wire message types (`types::apollo::*`, etc.) + shared traits
- `util/` -- bincode codec + ip-file loader
- `tools/genconfig/` -- generates X.509 certs + node/client configs
- `stress-test/` -- the benchmark harness described above
- `scripts/` -- shell drivers (retained for reproducibility)
- `docs/`, `data/`, `Plots/` -- paper artefacts (plots, raw runs)

## Citing

See [`CITATION.cff`](CITATION.cff).

## Tag

The state of the repository at the time the FC artefact was frozen is tagged
as `apollo-fc2023-artifact`.
