# Typing Simulator

A native Rust desktop app that types pasted text into another application at a human-like pace. It replaces the original Python/Tkinter app; Python is not needed to run it.

## Download and use

Download the latest file from [GitHub Releases](https://github.com/SinghZoren/TypingSimulator_Rust/releases):

- Windows x64: `TypingSimulator-windows-x64.zip` — extract and run `TypingSimulator.exe`.
- macOS (Intel or Apple Silicon): `TypingSimulator-macos-universal.zip` — extract and open `Typing Simulator.app`.

The setup guide opens as a modal on first launch. Use its Windows/macOS toggle for the instructions that match your computer; reopen it any time with **Setup guide** in the top bar. The main page scrolls when the window is small.

Paste or enter text, adjust the settings if desired, then press **Start typing** or the configured shortcut (`F6` by default). Focus the target window during the countdown. Press the shortcut again to stop. The app preserves whitespace, punctuation, and line breaks, and can insert then correct occasional typos. Optional thinking pauses happen only after a completed word. The default is 120 WPM with a three-second countdown.

The editor shows an approximate completion time before a run and approximate remaining time while typing. It includes the countdown, expected typo corrections, and expected thinking pauses; random choices mean the actual duration will vary.

To remap the shortcut, choose a function key or letter in the **Global shortcut** card, select any modifiers, and click **Apply shortcut**. Letter shortcuts require Ctrl, Alt, or Win/Cmd. The app keeps the previous shortcut if the new one cannot be registered. Speed, realism, onboarding status, and shortcut preferences are saved between launches; pasted text is not saved.

macOS requires **System Settings → Privacy & Security → Accessibility** permission for Typing Simulator. The download is currently unsigned, so on first launch you may need to right-click the app and choose **Open**. Windows may show a SmartScreen warning for the unsigned executable. Typing into a Windows app running as administrator requires starting Typing Simulator as administrator too.

## Build from source

Install [Rust](https://rustup.rs/) and run:

```sh
cargo run --release
```

The app uses `eframe` for its GUI, `global-hotkey` for the system-wide shortcut, and `enigo` for keyboard input. Settings are adjustable in the window: 20–250 WPM, 0–10 seconds of start delay, corrected typo chance, timing variation, thinking-pause chance, and pause duration. Each slider has an explanation in the app.

The UI bundles the [Inter font](assets/fonts/OFL.txt) under the SIL Open Font License.

## Publish a release

Push a version tag. GitHub Actions builds a Windows executable and a universal macOS app, then attaches both ZIP files to a GitHub Release:

```sh
git tag v1.0.0
git push origin v1.0.0
```

You can also start the workflow manually from the Actions tab. That creates downloadable workflow artifacts without publishing a release.

## Development checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
