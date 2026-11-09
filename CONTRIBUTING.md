# Contributing

Contributions are welcome. For a bug report, include the steps to reproduce
it, the expected and actual behavior, and relevant platform, Rust and GPU
information. For a feature or architectural change, describe the use case and
proposed behavior before investing in a large implementation.

## Get started

Install Rust through [rustup](https://rustup.rs/); the repository selects the
stable toolchain and its formatting and Clippy components. From the repository
root:

```sh
cargo test -p wxsl-core
cargo run -p wxsl --example pbr_cube
```

The example opens a window and needs a working GPU adapter. See the [README](README.md)
for other examples and [architecture](docs/architecture.md) for the crate
boundaries. [AGENTS.md](AGENTS.md) lists the conventions and routes changes to
the relevant tests; its guidance applies to human contributors too.

## Prepare a change

Keep each change focused and preserve the workspace boundaries. Update the
relevant documentation and examples when behavior changes. Add or update tests
for the behavior being changed. For a nontrivial public boundary, crate,
dependency or feature-matrix decision, add a short [architecture decision](docs/adr/README.md).

Start with the focused tests in [AGENTS.md](AGENTS.md), then run the checks
required by the affected boundary. The common workspace checks are:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

GPU tests can skip when no adapter is available; report a skip rather than
calling it a device pass. For changes to shaders, the shader ABI or pipelines,
render a picture where possible:

```sh
cargo run -p wxsl --example pbr_cube -- --headless
```

Forward and deferred images should agree within 8-bit rounding. The CI and
[AGENTS.md](AGENTS.md) describe the additional C ABI, Dawn, WebGL and public
API checks. A change to the stable API, document formats or C ABI also needs
an unreleased changelog entry and, where applicable, updated API snapshots.

## Open a pull request

Explain the problem and resulting behavior, list the checks run and note any
skipped device tests. Include images for a visible rendering change when they
help reviewers. Keep one concern per pull request; cross-crate changes are
welcome when the behavior crosses those boundaries.

## Licensing and shader sources

By contributing workspace code, you agree to the project's dual MIT / Apache-2.0
license. Shader ports are limited to audited MIT, Apache-2.0 or BSD-2/3-Clause
sources. Preserve the copyright, license and pinned provenance, and update the
notices as described in the [shader guide](crates/wxsl-stdlib/shaders/README.md).
Assets in `resources/` have their own licenses; check the notices beside them.
