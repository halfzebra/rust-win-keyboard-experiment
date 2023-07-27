# ⌨ Rust Windows Keyboard Experiment

This program is designed to automatically switch your keyboard layout based on the application you are currently using. By providing a configuration file in YAML format, you can specify which keyboard layout should be used for each application. The program will continuously monitor your active window and switch the layout accordingly. This can save you time and effort by eliminating the need to manually switch your layout every time you switch between different applications. Additionally, it can be useful for users who frequently switch between languages and want to avoid accidentally typing in the wrong one.

## Running Locally

Create a config file with language code to app mapping.

```yaml
en-US: 
  - Code.exe
  - steam.exe
  - HuntGame.exe
  - ubuntu2004.exe
uk-UA: 
  - Discord.exe
  - Telegram.exe
```

Compile the binary or run directly from source:

```bash
cargo run .\mocks\lang.yml
```

## Useful Links
- https://docs.microsoft.com/en-us/windows/win32/intl/language-identifiers
- https://winprotocoldoc.blob.core.windows.net/productionwindowsarchives/MS-LCID/%5bMS-LCID%5d.pdf
- https://microsoft.github.io/windows-docs-rs/doc/windows/
- https://stackoverflow.com/questions/51117874/how-to-send-wm-inputlangchangerequest-to-app-with-modal-window
