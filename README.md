<p align="center">
  <img src="ui/public/favicon.png" alt="JustBlast logo" width="88" height="88">
</p>

<h1 align="center">JustBlast</h1>

<p align="center">
  Read the board, explore your moves, and play your next Block Blast cycle.
</p>

<p align="center">
  <a href="#-why-justblast">About</a> |
  <a href="#-features">Features</a> |
  <a href="#-getting-started">Getting started</a> |
  <a href="#-developer-guide">Developer guide</a>
</p>

**JustBlast is a desktop companion for Block Blast.** Enter a board, find a sequence of placements, compare alternatives, and preview each move before playing.

Connect an Android phone to capture the game screen, calibrate its layout, and execute placements from the same workspace.

<br>

## 💡 Why JustBlast?

See how a sequence changes the board before committing to it. Review placement order and position, correct a misread piece, and continue planning from the moves already played.

Input, solving, previews, and phone controls share one workspace. The application starts empty, so you can enter a position yourself or scan a connected phone.

<br>

## ✨ Features

- **Board and piece editor.** Draw the board, choose from 42 catalogue shapes, rotate or flip pieces, and save custom shapes.
- **Move planning.** Solve the current position, inspect each step, and explore alternatives by placement order and position.
- **Local preview.** Play, pause, and reset a simulation without moving a phone. Continue with another simulated cycle from the resulting board.
- **Phone capture.** Read the board and tray through ADB. Review uncertain recognition and correct pieces directly in the workspace.
- **Visual calibration.** Adjust board and tray areas, preview recognition, test a placement, and save or import a hardware profile.
- **Phone execution.** Enable Move device to play on the phone, track acknowledged placements, and stop or continue between cycles.
- **Preferences and history.** Keep custom shapes, scoring weights, connection settings, and cycle records locally.

Board editing, solving, and previews work **without a phone**. Phone features require **Android platform-tools (`adb`)** and an authorized device.

The bundled capture helper targets ARM64 devices running Android API 24 or later.

<br>

## ⚙️ Built with

<p align="left">
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/Svelte_5-FF3E00?style=for-the-badge&amp;logo=svelte&amp;logoColor=white" alt="Svelte 5"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&amp;logo=rust&amp;logoColor=white" alt="Rust"></a>
  <a href="https://tauri.app/"><img src="https://img.shields.io/badge/Tauri_2-24C8D8?style=for-the-badge&amp;logo=tauri&amp;logoColor=black" alt="Tauri 2"></a>
  <a href="https://www.typescriptlang.org/"><img src="https://img.shields.io/badge/TypeScript_7-3178C6?style=for-the-badge&amp;logo=typescript&amp;logoColor=white" alt="TypeScript 7"></a>
</p>

Rust handles game rules, search, accepted state, and device execution. Svelte renders the interface, and Tauri packages the application as a desktop executable with its UI bundled inside.

<br>

## 🚀 Getting started

Follow these steps to run JustBlast from source. An installed release does not require Node.js, Bun, or Rust.

### Requirements

| Tool                   | Required version                                                                   |
| ---------------------- | ---------------------------------------------------------------------------------- |
| Node.js                | `26.13.1`                                                                          |
| Bun                    | `1.4.4`                                                                            |
| Rust                   | `1.100.0`, with `clippy` and `rustfmt`                                             |
| Desktop dependencies   | [Tauri prerequisites for your platform](https://v2.tauri.app/start/prerequisites/) |
| Android platform-tools | Required for phone features; `adb` must be on `PATH`                               |

Toolchain versions are pinned in [`.node-version`](.node-version), [`.bun-version`](.bun-version), and [`rust-toolchain.toml`](rust-toolchain.toml). Commit both dependency lockfiles when updating dependencies.

### 1. Prepare the environment

On Ubuntu, install the desktop build dependencies used by this project's CI:

```bash
sudo apt-get update
sudo apt-get install --no-install-recommends -y libwebkit2gtk-4.1-dev build-essential librsvg2-dev patchelf
```

For other distributions or Windows, follow the platform prerequisites linked above.

From the project directory, install dependencies:

```bash
bun install --frozen-lockfile
```

### 2. Start JustBlast

```bash
bun run dev
```

This opens the desktop application and starts Vite for development.

Enter the current board and pieces, then use Search to calculate a plan. Keep Move device off to preview placements locally.

### 3. Connect a phone

1. Enable USB debugging on the Android phone, connect it, and authorize the computer.
2. Open **Settings → Phone connection** and use **Discover** to find the device. Wireless pairing and connection are also available in this section.
3. Use **Phone calibration** to check the board and tray alignment, test a placement, and save a profile for your phone.
4. Connect the device and use **Scan** to read its board and pieces. Review uncertain results before playing.
5. Enable **Move device** when you want Play to execute placements on the phone.

The included `default` profile is a starting layout for portrait 1080 × 2400 screens. Check calibration against your actual game screen. Scanning and searching do not move the phone; the calibration test and Play on phone do.

<br>

## 🔧 Developer guide

<details>
<summary><strong>Development and verification</strong></summary>

### Development commands

| Command                      | Purpose                                                                     |
| ---------------------------- | --------------------------------------------------------------------------- |
| `bun run dev`                | Start the Tauri desktop application with Vite                               |
| `bun run dev:ui`             | Start the browser preview on `http://127.0.0.1:1420`                        |
| `bun run typecheck`          | Check Svelte, TypeScript, and tooling types                                 |
| `bun run lint`               | Check JavaScript, architecture boundaries, asset baselines, and Rust Clippy |
| `bun run format:check`       | Check Prettier and Rust formatting                                          |
| `bun run format`             | Apply formatting                                                            |
| `bun run contracts:generate` | Generate frontend IPC types from Rust models                                |
| `bun run contracts:check`    | Check that generated IPC types match Rust models                            |
| `bun run test`               | Run frontend unit tests and offline Rust tests                              |
| `bun run check`              | Run types, lint, formatting, contracts, and offline tests                   |
| `bun run test:e2e`           | Run browser tests through the Rust test transport                           |
| `bun run test:desktop`       | Run the Linux release window smoke test through WebKit WebDriver            |
| `bun run benchmark`          | Measure the Rust solver and move explorer                                   |

Browser preview reports when the native desktop connection is unavailable. Use the desktop application for solving and phone operations, or the test transport for automated browser tests.

### Browser tests

Install Chromium, then run the suite:

```bash
node node_modules/@playwright/test/cli.js install chromium
bun run test:e2e
```

Tests exercise the Rust services through a test-only IPC transport. Phone tests use fake devices and do not call real ADB. Visual reference tests require a separately running reference application through `JUSTBLAST_LEGACY_URL`; they are skipped when it is absent.

The Linux desktop smoke test requires a release build, a running WebKit WebDriver, and an isolated display and application data directory. Its driver endpoint defaults to `http://127.0.0.1:4445` and can be set with `JUSTBLAST_WEBDRIVER_URL`. `JUSTBLAST_NATIVE_BINARY` selects the executable to test.

### Continuous integration

The [verification workflow](.github/workflows/ci.yml) is configured for Ubuntu and Windows. It runs project checks, browser tests, and a desktop build, then uploads the installer artifacts.

Linux builds and offline tests do not establish Windows or real-phone validation. Check recognition, calibration, and touch placement on the intended hardware before relying on phone execution.

</details>

<br>

<details>
<summary><strong>Architecture and execution</strong></summary>

### Project structure

| Directory                   | Responsibility                                                         |
| --------------------------- | ---------------------------------------------------------------------- |
| `crates/justblast-core`     | Pure board rules, shapes, scoring, solver, and workspace state         |
| `src-tauri/src/api`         | Typed IPC requests, responses, and command mapping                     |
| `src-tauri/src/application` | Calibration, input, and phone workflows                                |
| `src-tauri/src/device`      | ADB transport, recognition, calibration profiles, and touch execution  |
| `ui/src/features`           | Workspace, editor, plan explorer, and Settings controllers             |
| `ui/src/components`         | Shared presentation components                                         |
| `ui/src/lib/native`         | Frontend IPC calls and response validation                             |
| `ui/src/generated`          | TypeScript contracts generated from Rust                               |
| `resources`                 | Shape catalogue, anonymous default profile, and Android capture helper |
| `tests`                     | Frontend unit tests, browser tests, fixtures, and desktop smoke tests  |

### Execution behavior

Rust owns accepted input and physical progress. Completed phone placements come from backend acknowledgements and remain locked when previewing, correcting input, or replacing the remaining plan.

Move device controls whether Play previews locally or executes on the phone. Enabling it does not start movement. The application starts without reconnecting or resuming phone execution.

Stop finishes the authorized batch and prevents the next one. Force stop interrupts the local channel and requires recovery before continuing. An uncertain touch is not automatically retried.

</details>

<br>

<details>
<summary><strong>Local data and device profiles</strong></summary>

Preferences, saved profiles, history, execution recovery records, and diagnostics use the operating system's application data directory for `com.justblast.desktop`.

The [Git ignore rules](.gitignore) exclude private profiles, exported TOML files, device settings, local environment files, backups, captures, diagnostics, and ADB keys. The anonymous [`default` profile](resources/default-profile.toml) is the public built-in preset. Keep personal calibrations separate from that file.

Hardware profile exports contain screen, touch, and timing settings. They exclude connection addresses, device serials, scoring weights, and automation preferences.

</details>

<br>

<details>
<summary><strong>Release builds</strong></summary>

### Build an installer

```bash
bun run check
bun run build
```

The build bundles the UI into the desktop executable and creates an installer for the host platform:

| Platform | Installer               | Output directory              |
| -------- | ----------------------- | ----------------------------- |
| Linux    | Debian package (`.deb`) | `target/release/bundle/deb/`  |
| Windows  | NSIS installer (`.exe`) | `target/release/bundle/nsis/` |

Build each installer on its corresponding platform. Use `bun run build:ui` to build only the frontend assets in `dist/`.

</details>
