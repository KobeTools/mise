# Local Development Guide - mise

This guide covers the chronological setup, development workflow, and verification steps required to contribute to `mise` locally.

---

## 1. Environment Setup (macOS)

### System Dependencies
```sh
xcode-select --install
brew install git
curl [https://sh.rustup.rs](https://sh.rustup.rs) -sSf | sh
rustup default stable
```

### Build from Source
```sh
git clone [https://github.com/KobeTools/mise.git](https://github.com/KobeTools/mise.git)
cd mise
cargo build --all-features
cargo install --path .
```
> *Note: The first build bootstraps a number of external tools, so it can take a while.*

### Shell Configuration & Path Tuning
To use your local build efficiently and prevent duplicate PATH entries, configure your shell to use `mise`'s dynamic activation rather than static shims. Add the following to your `~/.zshrc`:

```zsh
# Ensure $path is treated as an array of unique values
typeset -U path
path=("$HOME/.local/bin" "$HOME/.cargo/bin" $path)
export PATH

# Official Mise Activation (dynamic PATH rewriting)
if command -v mise >/dev/null; then
  eval "$(mise activate zsh)"
fi
```

### Verification
Reload your shell (`source ~/.zshrc`), then verify you are running the local binary:
```sh
command -v mise
mise doctor
```
For an optimized macOS build, `doctor` should output: `activated: yes` and `shims_on_path: no`.

---

## 2. Environment Setup (Windows)

### System Compilers & Toolchain
Rust on Windows requires the MSVC linker. Install the headless build tools:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install Git.Git
winget install Rustlang.Rustup
rustup default stable
```

### Build from Source
```powershell
git clone [https://github.com/KobeTools/mise.git](https://github.com/KobeTools/mise.git)
cd mise
cargo build --all-features
cargo install --path .
```
> *Note: Windows builds may take longer due to NTFS overhead and antivirus scanning during the bootstrap phase.*

### Compiling the Native Shim
Unlike official installers, `cargo install` only moves the main `mise.exe`. To prevent structural warnings in `mise doctor`, manually compile and move the native Windows shim helper:

```powershell
cargo build --release -p mise-shim
copy .\target\release\mise-shim.exe $HOME\.cargo\bin\
```

### PATH Priority & Profile Activation
1. **Path Priority:** Move `C:\Users\<USER>\.cargo\bin` to the very top of your user `Path` environment variable. This ensures Windows finds your local build before it hits any shim folders.
2. **Profile Setup:** Add the dynamic activation hook to your PowerShell `$PROFILE`:

```powershell
if (Get-Command mise -ErrorAction SilentlyContinue) {
    mise activate pwsh | Out-String | Invoke-Expression
}
```

### Verification
Restart your PowerShell terminal and initialize the shims for the first time:

```powershell
mise reshim
where.exe mise
mise doctor
```
`where.exe` should list your `.cargo\bin` path first, and `doctor` should report **"No problems found"**.

---

## 3. Development Workflow

Once your environment is built and activated, use these tools to iterate on the codebase.

### Fast Local Iteration
Use the cargo runner for quick logic checks without reinstalling the binary globally:
```sh
cargo run -- ls
```

### Install the Local Git Pre-Commit Hook
`mise` defines a `pre-commit` task that runs `mise run lint`. Install this hook so it runs automatically before you commit code:
```sh
mise generate git-pre-commit --write --task=pre-commit
```

---

## 4. Pre-Push Validation

### Lint Before You Push
The GitHub `lint` job runs `mise run lint`. If that fails on `cargo-fmt`, fix the formatting locally and re-run the full lint task:

```sh
cargo fmt --all
mise run lint
```

### Commands That Best Match GitHub Checks
Run these specific tests locally before relying on CI:

```sh
cargo test --all-features
./e2e/run_test cli/test_use
./e2e/run_test cli/test_unuse
./e2e/run_test tasks/test_task_ls
./e2e/run_test tasks/test_task_ubi_autoinstall
```

If you want broader coverage, run the full E2E suite:

**macOS/Linux:**
```sh
mise run test:e2e
```

**Windows (PowerShell):**
```powershell
cargo test --all-features
pwsh e2e-win\run.ps1
pwsh e2e-win\run.ps1 task
```