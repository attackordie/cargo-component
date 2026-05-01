# wit-bindgen Migration: Legacy Split-Crate to Unified 0.55+

This is the runtime-side migration any workspace must perform once it installs
this fork (see `FORK.md`). The fork's bindings generator emits paths under
`wit_bindgen::rt::*` (unified crate, 0.55+). The legacy split-crate shape —
`wit-bindgen = "0.41"` plus a separate `wit-bindgen-rt` dep — produces
`E0282` / `E0433` errors in the generated `bindings.rs`
(`could not find Cleanup in rt`, type-inference failures, etc).

This is a contract bump, not a behavior change. After migration, sync
components compile identically; async-exporting components newly compile.

This document is fork-only and is NOT intended to be submitted upstream
alongside the PR-candidate commits on `fix/wit-bindgen-rt-unified-runtime-path`.

## 1. Detect whether you need the migration

From the workspace root:

```bash
# List every Cargo.toml that pins a runtime wit-bindgen version below 0.55
grep -rE '^wit-bindgen(-rt)?\s*=' --include=Cargo.toml .

# Symptom on attempted build:
#   error[E0433]: could not find `Cleanup` in `rt`
#   error[E0282]: type annotations needed
# in target/.../out/bindings.rs against the fork's generator output.
```

You need the migration if any line shows:
- `wit-bindgen = "0.4x"` (pre-0.55), OR
- `wit-bindgen-rt = "..."` (separate runtime crate at any version).

Workspace pins inherit transparently: if your workspace `Cargo.toml` pins
`wit-bindgen` and member crates only have `wit-bindgen.workspace = true`,
you only edit one place.

## 2. The mechanical change

### Before (split-crate, legacy)

```toml
[dependencies]
wit-bindgen = "0.41.0"
wit-bindgen-rt = { version = "0.41.0", features = ["bitflags"] }
```

### After (unified, sync component)

```toml
[dependencies]
wit-bindgen = { version = "0.56", features = ["bitflags"] }
# wit-bindgen-rt is REMOVED. The unified crate exposes `rt` as a submodule.
```

### After (unified, async-exporting component)

For components that **export** async Guest traits — e.g. `wasi:http/handler@0.3.0-rc-2026-03-15`:

```toml
[dependencies]
wit-bindgen = { version = "0.56", features = ["bitflags", "async", "async-spawn"] }
```

The async features are caller-opt-in. Importing async APIs does not require
them; only exporting async traits does. If you are unsure, start with
`["bitflags"]` and add `async` features only if the build complains about
missing `async_support` items.

## 3. One pin, not N pins

Pin at the **workspace root**, then have member crates use
`wit-bindgen.workspace = true`. Avoid per-crate version pins — a workspace
that ends up with multiple `wit-bindgen` versions in `Cargo.lock` will
compile but produce non-interoperable types (two `Cleanup` structs from two
different crates are not the same type, even if structurally identical).

```toml
# Workspace root Cargo.toml
[workspace.dependencies]
wit-bindgen = { version = "0.56", features = ["bitflags"] }
```

```toml
# Each component crate
[dependencies]
wit-bindgen.workspace = true
```

A component that needs the async features overrides locally:

```toml
[dependencies]
wit-bindgen = { workspace = true, features = ["bitflags", "async", "async-spawn"] }
```

This keeps a single version in `Cargo.lock` while letting individual crates
extend the feature set.

## 4. Auto-generated bindings regenerate

`cargo-component` regenerates `bindings.rs` on every build; you do not need
to delete or edit them manually. The first build after migration will show
fresh codegen output referencing `wit_bindgen::rt::*`.

If your component checks `bindings.rs` into source control (uncommon but
possible), regenerate it via a clean build:

```bash
cargo clean
cargo component build --release
```

## 5. Verify

```bash
# 1. Confirm only one wit-bindgen version resolves
cargo tree -p <component-name> -i wit-bindgen
#    (should report a single 0.56.x entry)

# 2. Confirm the legacy crate is gone
cargo tree -p <component-name> -i wit-bindgen-rt
#    (should report "package ID specification did not match any packages")

# 3. Build the workspace
cargo component build --release --workspace
#    (every component should compile; bindings.rs references wit_bindgen::rt::*)

# 4. Validate one emitted .wasm against the component-model async feature
wasm-tools validate --features cm-async target/wasm32-wasip2/release/<component>.wasm
```

If step 3 fails with `could not find Cleanup in rt`, recheck step 1 — a
member crate likely still pins a sub-0.55 version directly.

## 6. Cross-reference

- Why this fork pins the contract: `FORK.md` in this repository (specifically
  commit `93cdd3d` setting `runtime_path = "wit_bindgen::rt"` in
  `src/bindings.rs`).
- A consumer workspace that already maintains a regression suite against this
  fork: `attackordie/OCI_Registry`. Its `push-automation/docs/CARGO_COMPONENT_FORK.md`
  has install + verification recipes; its `tests/test-components/p3-http/Cargo.toml`
  is a reference async-exporting component already on the unified shape; its
  `push-automation/src/templates.rs` and `push-automation/src/cage/build.rs`
  carry revert guards locking the unified-crate template shape.
