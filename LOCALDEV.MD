# Local Development Guide - mise

This guide explains how to set up and maintain a local development environment for the `mise` repository on Windows.

---

## 🏁 Windows Setup
*Perform these steps to ensure your environment can handle the `mise` build scripts.*

### 1. System Compilers & Toolchain
Rust on Windows requires the MSVC linker. Use WinGet to install the headless build tools:
```powershell
# Install C++ Build Tools
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# Install Rust
winget install Rustlang.Rustup
rustup default stable
```

### 2. System Helpers
```powershell
# Install Git
winget install Git.Git
```

---

## 🚀 Building from Source

### 1. Clone the Repository
```powershell
git clone [https://github.com/jdx/mise.git](https://github.com/jdx/mise.git)
cd mise
```

### 2. Build and Install
Running this command will compile the binary in release mode and move it to your global Cargo directory (`~/.cargo/bin`).
```powershell
cargo install --path .
```
> **Note:** The first build triggers a bootstrap phase where `mise` downloads ~25 external tools (like `jq`, `gh`, etc.). This can be slow on Windows due to NTFS file I/O and real-time antivirus scanning.

---

## ⚠️ Windows-Specific Troubleshooting

### 1. The PATH Priority Conflict (Critical)
If you already have a version of `mise` installed via WinGet or a separate installer, it uses **Shims** to manage commands. If you build `mise` manually, the existing shims might still point to the old version or fail with `cannot find binary path`.

**The Fix:** 1. Open your Environment Variables.
2. Move `C:\Users\<USER>\.cargo\bin` to the **very top** of your User Path. 
3. This ensures Windows finds your locally built `mise.exe` before it hits the `mise` shims folder.

### 2. Development "Fast" Check
If you are making small changes to the code and don't want to wait for a 10-minute release build, use the debug runner:
```powershell
cargo run -- ls
```
This is much faster for testing logic changes but does not install the binary globally.

---

## 🔍 Verification
After the installation finishes, open a **new** terminal window and run:
```powershell
where.exe mise
mise --version
```
*The version date should reflect the current day of your build.*