<h3 align="center">
  <img src="icon.png" width="120" alt="YAAM Logo"/><br/>
  <img src="https://raw.githubusercontent.com/catppuccin/catppuccin/main/assets/misc/transparent.png" height="30" width="0px"/>
  YAAM - Yet Another Archive Manager
  <img src="https://raw.githubusercontent.com/catppuccin/catppuccin/main/assets/misc/transparent.png" height="30" width="0px"/>
</h3>

<p align="center">
  <b>A modern, high-performance file archiver written in Rust.</b><br/>
  Archive manager with a proprietary format, dark mode UI, and native Windows integration.
</p>

<p align="center">
  <a href="https://github.com/knotorganization/YAAM"><img src="https://img.shields.io/badge/built_with-Rust-orange?style=for-the-badge&logo=rust"></a>
  <img src="https://img.shields.io/badge/version-0.2.0-blueviolet?style=for-the-badge">
  <img src="https://img.shields.io/badge/license-GPLv3-success?style=for-the-badge">
</p>

---

## ✨ What is YAAM?

**YAAM** is a next-generation file archiver. It introduces the **.yaam** format—a secure, deduplicated container that uses modern compression standards.

- ✅ **Smart Preview:** Peek inside archives without extracting.
- 🎨 **Modern UI:** Dark Mode by default.
- 🔒 **Secure:** Custom XOR obfuscation + Zstandard encryption.
- ⚡ **Native:** Right-click context menu integration.

---

## 📦 Features

- 📝 **Multi-Format Support:**
  - **Create:** `.yaam` (Proprietary), `.zip`
  - **Extract:** `.yaam`, `.zip`, `.7z`
- 🧠 **Smart Context Menu:** Right-click any folder to "Pack to .yaam".
- 🔨 **High Performance:** Powered by Facebook's **Zstandard** algorithm (faster than Deflate).
- ⚡ **Zero-Install Preview:** View file contents instantly in a table view.

---

## 🚀 Installation

### Windows Installer (Recommended)

1. Go to the [**Releases**](https://github.com/knotorganization/YAAM/releases) page.
2. Download `YAAM_Setup.exe`.
3. Run the installer to set up the **Context Menu** and file associations automatically.

### Build from Source

Requirements: **Rust** and **C++ Build Tools** (MSVC).

```bash
# 1. Clone the repository
git clone https://github.com/knotorganization/YAAM.git
cd yaam

# 2. Build the release binary
cargo build --release

# 3. (Optional) Run the Setup Script manually if not using the Installer
./install_context_menu.ps1
