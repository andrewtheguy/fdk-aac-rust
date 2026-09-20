# What this repository is

An **unmodified mirror** of the Fraunhofer FDK AAC decoder in Rust as it ships in
AOSP, so that it can be used as a Cargo git dependency. The crate is not published
on crates.io; nothing here is a fork.

Everything beside this file is upstream, byte for byte:

- Upstream: `platform/external/aac`, subdirectory `rust/`
- Tag: `android-17.0.0_r1`
- Commit: `41f344ffc0bacea87cac5bb1756bd40761265d1e`
- Source: <https://android.googlesource.com/platform/external/aac/+/refs/tags/android-17.0.0_r1/rust>

The repository root *is* that `rust/` directory, so the root package is the `aac`
crate and a dependency reads:

```toml
aac = { git = "https://github.com/andrewtheguy/fdk-aac-rust", tag = "android-17.0.0_r1" }
```

The workspace members `ffi/` (C bindings) and `framework/` (a command-line decoder
example) are carried because the root manifest names them; a dependent builds
neither.

## Updating to a later AOSP tag

Replace the tree and tag it after the AOSP tag, keeping this file:

```sh
curl -o rust.tar.gz 'https://android.googlesource.com/platform/external/aac/+archive/refs/tags/<TAG>/rust.tar.gz'
```

Do not edit the sources. The FDK AAC licence requires a modified version to be
renamed "Third-Party Modified Version of the Fraunhofer FDK AAC Codec Library for
Android" and to carry change notices; keeping the mirror verbatim is what avoids
that, and what keeps the next update a re-extract rather than a merge.

## Licence

`NOTICE` — the "Software License for The Fraunhofer FDK AAC Codec Library for
Android". It is not OSI-approved and grants no patent licence.
