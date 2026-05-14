# Waltzing Runtime

This repository is the companion workspace for Waltzing template tooling. It
currently contains a showcase server, generated library metadata, editor
extension assets, and the `waltzing-ui` template library.

## Contents

- `libraries/waltzing-ui` - shadcn-inspired Waltzing component library.
- `src/main.rs` - Axum showcase server for browsing components and examples.
- `build.rs` - discovers template libraries, validates manifests, and compiles
  them with the Waltzing CLI.
- `src/generated` - generated library metadata used by the showcase.
- `extensions` - editor extension work in progress.
- `releases` - release artifacts and packaging work in progress.

## Building

Install or build a recent `waltzing` CLI, then run:

```bash
cargo test --locked
```

The build script searches `PATH`, then `~/.local/bin`. To force a specific
compiler, set `WALTZING_BIN`:

```bash
WALTZING_BIN=/path/to/waltzing cargo test --locked
```

Template compilation is a hard build gate. Missing manifest paths, unknown
manifest dependencies, or Waltzing parse errors fail the build.

## Browser Regression Tests

Install the browser-test dependencies once:

```bash
npm install
npx playwright install chromium
```

Then run the browser and accessibility regression suite:

```bash
npm run test:browser
```

The Playwright suite starts the Axum showcase, visits the `waltzing-ui` library
and stable block routes in desktop and mobile Chromium, and runs axe checks
against rendered block previews.

## Running the Showcase

```bash
cargo run --locked
```

The showcase serves the component browser and static assets through Axum.

## Waltzing UI

See `libraries/waltzing-ui/README.md` for component usage, security boundaries,
and production setup notes.

## Issue Tracking

This project uses `bd` (beads). Run `bd prime` for current workflow context.
If the beads database is not present in a fresh checkout, run `bd onboard` for
setup instructions.

## License

MIT
