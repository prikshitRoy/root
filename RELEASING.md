# Release & Distribution Guide (`root`)

This document describes the official binary and installer formats distributed for **🌿 root**, how end users can install and verify each package, and how maintainers cut a new release.

---

## 1. Official Executable & Installer Formats

Every tagged release (`v*`) builds and publishes native installers and portable executables across Windows, macOS, and Linux via GitHub Actions (`.github/workflows/release.yml`):

| Platform | Architecture | Format | Filename | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Windows** | `x86_64` | **Setup Wizard (`.exe`)** | `root-setup-windows-x64.exe` | Graphical & CLI installer. Installs to `%LOCALAPPDATA%\Programs\root`, creates `root-leaf.ico`, Start Menu & Desktop `.lnk` shortcuts, and registers the Windows Uninstall entry. |
| **Windows** | `x86_64` | **Portable App (`.exe`)** | `root-windows-x64.exe` | Standalone portable desktop executable (`root.exe`). Runs directly without installation. |
| **Windows** | `x86_64` | **Portable Archive (`.zip`)** | `root-windows-x64.zip` | Contains both `root.exe` (portable app) and `root-installer.exe` (Setup Wizard). |
| **macOS** | `aarch64` (Apple Silicon) | **Disk Image (`.dmg`)** | `root-macos-arm64.dmg` | Standard macOS disk image containing `root.app`, `root-installer`, and an `Applications` symlink. |
| **macOS** | `aarch64` (Apple Silicon) | **App Bundle (`.app.zip`)** | `root-macos-arm64.app.zip` | Zipped `root.app` bundle for direct extraction into `/Applications` or `~/Applications`. |
| **macOS** | `aarch64` (Apple Silicon) | **Portable Tarball (`.tar.gz`)** | `root-macos-arm64.tar.gz` | Contains standalone `root` and `root-installer` binaries. |
| **macOS** | `x86_64` (Intel) | **Disk Image (`.dmg`)** | `root-macos-x64.dmg` | Standard macOS disk image for Intel Macs. |
| **macOS** | `x86_64` (Intel) | **App Bundle (`.app.zip`)** | `root-macos-x64.app.zip` | Zipped `root.app` bundle for Intel Macs. |
| **macOS** | `x86_64` (Intel) | **Portable Tarball (`.tar.gz`)** | `root-macos-x64.tar.gz` | Contains standalone `root` and `root-installer` binaries for Intel Macs. |
| **Linux** | `x86_64` (`amd64`) | **Debian Package (`.deb`)** | `root-linux-amd64.deb` | Native package for Ubuntu, Debian, Linux Mint, and Pop!_OS. Installs `/usr/bin/root`, `/usr/bin/root-installer`, `.desktop` launcher, and `root.svg`. |
| **Linux** | `x86_64` | **Setup Wizard (Binary)** | `root-installer-linux-x64` | Distro-agnostic graphical/CLI installer. Installs per-user into `~/.local/share/root`, `~/.local/bin/root`, and `~/.local/share/applications/root.desktop`. |
| **Linux** | `x86_64` | **Portable Tarball (`.tar.gz`)** | `root-linux-x64.tar.gz` | Contains portable `root` and `root-installer` executables. |

---

## 2. End-User Installation Instructions

### Windows (10 / 11 x64)

- **Recommended (Setup Wizard):**
  Download and run `root-setup-windows-x64.exe` to open the graphical Setup Wizard, or install silently from PowerShell/Command Prompt:
  ```powershell
  .\root-setup-windows-x64.exe --install
  ```
- **Portable (No Install):**
  Download `root-windows-x64.exe` and double-click it to launch `root` directly.

### macOS (Apple Silicon & Intel)

> **Note for macOS Users:** While Windows and Linux (Ubuntu) releases are manually tested on local environments, macOS packages (`.dmg`, `.app.zip`, and `.tar.gz`) are compiled and tested exclusively via GitHub Actions CI (`macos-latest`) as the maintainer does not currently have access to Mac hardware. Community feedback and bug reports from macOS users are welcome!

- **Recommended (`.dmg` Disk Image):**
  1. Download `root-macos-arm64.dmg` (M1/M2/M3/M4) or `root-macos-x64.dmg` (Intel).
  2. Double-click the `.dmg` file and drag `root.app` into the `Applications` folder.
  3. On first launch (unsigned open-source build), right-click `root.app` and select **Open**, or clear the quarantine attribute in Terminal:
     ```bash
     xattr -cr /Applications/root.app
     ```

### Linux (x64)

- **Ubuntu / Debian / Mint (`.deb` package):**
  ```bash
  sudo apt install ./root-linux-amd64.deb
  ```
- **Any Linux Distribution (Universal Setup Wizard):**
  ```bash
  chmod +x ./root-installer-linux-x64
  # Launch graphical Setup Wizard:
  ./root-installer-linux-x64
  # Or perform a headless per-user install:
  ./root-installer-linux-x64 --install
  ```

---

## 3. Verifying Binary Checksums (`SHA256SUMS.txt`)

Every GitHub Release includes `SHA256SUMS.txt`. Always verify your download in the same directory:

```bash
# Linux
sha256sum -c SHA256SUMS.txt --ignore-missing

# macOS
shasum -a 256 -c SHA256SUMS.txt --ignore-missing

# Windows (PowerShell)
Get-FileHash .\root-setup-windows-x64.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

---

## 4. Maintainer Release Checklist

Follow these steps to publish a new version of `root`:

1. **Update the Version Number:**
   Update `version = "X.Y.Z"` in `Cargo.toml` and run `cargo check` so `Cargo.lock` updates cleanly.

2. **Run the Full Test Suite Locally:**
   ```bash
   cargo test
   ```

3. **Commit the Version Bump:**
   ```bash
   git add Cargo.toml Cargo.lock
   git commit -m "chore(release): bump version to vX.Y.Z"
   git push origin main
   ```

4. **Create and Push the Release Tag:**
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```

5. **Verify the GitHub Actions Pipeline:**
   - Open the **Actions** tab on GitHub and monitor the **Release Cross-Platform Installers** workflow.
   - Once all 4 matrix targets (`windows-x64`, `macos-arm64`, `macos-x64`, `linux-x64`) pass `cargo test` and package their installers, the `publish-release` job will automatically create the public GitHub Release with `SHA256SUMS.txt` and the download table.

## 5. Release History

| Version | Date | Notes |
| ------- | ---- | ----- |
| v0.1.1 | 2026-11-25 | Windows Installed apps logo and uninstall fixes; Linux launcher icon fixes. |
| v0.1.0 | - | First multi-OS release. |
