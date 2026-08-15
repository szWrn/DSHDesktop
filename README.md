# DSH Desktop

> A lightweight desktop shell for [DeepSeek Harness (DSH)](https://www.npmjs.com/package/@deepseek-ai/dsh): start the service with one click and keep it in your system tray — no terminal needed.

[中文文档](./README.zh-CN.md)

[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows&logoColor=white)](https://github.com/szWrn/DSHDesktop)
[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri&logoColor=black)](https://tauri.app)
[![Vue](https://img.shields.io/badge/Vue-3-42b883?logo=vue.js&logoColor=white)](https://vuejs.org)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.txt)

DSH Desktop is an unofficial, third-party desktop client built with Tauri 2 + Vue 3. It launches the DSH web service locally and embeds the web UI in a native window, so you can use DeepSeek Harness like a local app.

---

## Features

- **One-click start**: Launches the DSH service automatically and polls until it is ready — no manual `npx` commands.
- **One-click stop**: Locates and terminates the DSH process listening on port 3080.
- **System tray**: Closing the window minimizes it to the tray; the app keeps running in the background and lets you toggle the service from the tray menu.
- **Health monitoring**: A status dot reflects the service state in real time, and unexpected crashes are detected.
- **System locale**: Follows your system language to switch between Chinese and English UI.
- **Frameless window**: Ships with its own minimize / maximize / close buttons for a cleaner look.

---

## Screenshot

> Please add a screenshot or GIF (e.g. `docs/screenshot.png`) showing the full flow: start → service online → embedded DSH UI.

<!--
![DSH Desktop](docs/screenshot.png)
-->

---

## Getting Started

### Prerequisites

- **Windows** (currently Windows-only)
- **Node.js** (`npx` must be available on PATH; on first start the app pulls `@deepseek-ai/dsh` via `npx`, so a network connection is required)
- Ensure `127.0.0.1:3080` is free

> **Platform note**: The backend process management currently relies on `taskkill` / `netstat`, so only Windows is supported. macOS / Linux support is planned — contributions welcome.

### Use the binary

Download the latest installer from [Releases](../../releases) and double-click to install.

### Build from source

```bash
# 1. Install frontend dependencies
npm install

# 2. Run in development mode (Tauri window + HMR)
npm run tauri dev

# 3. Build the installer (output: src-tauri/target/release/bundle)
npm run tauri build
```

---

## Usage

1. Open the app and click **Start**: it launches the local DSH service and embeds `http://127.0.0.1:3080` into the window.
2. Click **Stop** to terminate the DSH service.
3. Click the **close** button to hide the window to the system tray (the app does not exit).
4. Right-click the tray icon to **Show / Start·Stop service / Quit**. Choosing **Quit** also cleans up the DSH process.

---

## How it works

```
┌─────────────────────────────────────────────┐
│                 DSH Desktop                 │
│  ┌─────────────┐   invoke    ┌───────────┐  │
│  │  Vue 3 UI   │ ──────────▶ │ Rust      │  │
│  │  (iframe)   │ ◀────────── │ backend   │  │
│  └─────────────┘ state/cmds  └─────┬─────┘  │
│        ▲                           │ spawn  │
│        │ embeds                   ▼        │
│  ┌─────────────────────────────────────┐    │
│  │  DSH web service (127.0.0.1:3080)   │    │
│  │  npx --yes @deepseek-ai/dsh web     │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

- **Start**: checks whether `127.0.0.1:3080` is free → silently runs `npx --yes @deepseek-ai/dsh web` (no console window on Windows) → polls until the port is ready.
- **Stop**: finds the process listening on port 3080 via `netstat -ano` and terminates it with `taskkill`.
- **Status**: the UI calls `is_dsh_service_running` every 3 seconds to drive the status dot and tray menu text.

---

## FAQ

**Q: Port 3080 is already in use?**
A DSH service (or another process) is already running. Free the port first, or click **Stop** and then **Start** again.

**Q: Nothing happens / start fails?**
Make sure Node.js is installed and `npx` works (verify with `npx --version`), and stay online so `@deepseek-ai/dsh` can be fetched.

**Q: Does it support macOS / Linux?**
Not yet. The current version is Windows-only; cross-platform support is on the roadmap.

**Q: Is this an official app?**
No. DSH Desktop is a third-party community project and is not affiliated with DeepSeek.

---

## Tech Stack

| Layer       | Technology                                          |
| ----------- | --------------------------------------------------- |
| Desktop     | [Tauri 2](https://tauri.app) (Rust)                 |
| Frontend    | [Vue 3](https://vuejs.org) + [Vite](https://vitejs.dev) |
| i18n        | `rust-i18n` + Vue frontend strings                  |
| Process     | Rust `std::process` + `netstat` / `taskkill`        |

---

## Contributing

Issues and pull requests are welcome. Read the build steps above and verify `npm run tauri dev` works locally before submitting.

Good first issues: macOS / Linux process-management port, installer icon & metadata polish, automated release CI.

---

## License

[MIT](LICENSE.txt) © 2026 [szWrn](https://github.com/szWrn)
