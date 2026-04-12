# Local Development Guide - mise

This guide covers the local setup and verification steps that are most useful
before pushing changes to `mise`.

## Install the Local Git Pre-Commit Hook

`mise` already defines a `pre-commit` task that runs `mise run lint`. Install
the local git hook once per clone:

```sh
mise generate git-pre-commit --write --task=pre-commit
```

That writes `.git/hooks/pre-commit` and runs the lint task every time you
commit.

## macOS Setup

### System dependencies

```sh
xcode-select --install
brew install git
curl https://sh.rustup.rs -sSf | sh
rustup default stable
```

### Build from source

```sh
git clone https://github.com/jdx/mise.git
cd mise
cargo build --all-features
cargo install --path .
```

The first build bootstraps a number of external tools, so it can take a while.

### Fast local iteration

```sh
cargo run -- ls
```

Use this for quick logic checks without reinstalling the binary globally.

## Windows Setup

### System compilers and toolchain

Rust on Windows requires the MSVC linker. Use WinGet to install the headless
build tools:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install Git.Git
winget install Rustlang.Rustup
rustup default stable
```

### Build from source

```powershell
git clone https://github.com/jdx/mise.git
cd mise
cargo build --all-features
cargo install --path .
```

The first build bootstraps a number of external tools, so it can take a while,
especially on NTFS or when antivirus is scanning new files.

### PATH priority conflict

If you already have `mise` installed from WinGet or another installer, move
`C:\Users\<USER>\.cargo\bin` to the top of your user `Path` so your local build
is found before any older shim-based install.

### Fast local iteration

```powershell
cargo run -- ls
```

## Lint Before You Push

The GitHub `lint` job runs `mise run lint`. If that fails on `cargo-fmt`, fix
the formatting locally and re-run the full lint task:

```sh
cargo fmt --all
mise run lint
```

On Windows, run the same commands from PowerShell:

```powershell
cargo fmt --all
mise run lint
```

## Commands That Best Match GitHub Checks

These are the first commands to run before relying on CI:

```sh
cargo test --all-features
./e2e/run_test cli/test_use
./e2e/run_test cli/test_unuse
./e2e/run_test tasks/test_task_ls
./e2e/run_test tasks/test_task_ubi_autoinstall
```

If you want broader coverage, run the full E2E suite:

```sh
mise run test:e2e
```

Windows has its own PowerShell E2E runner:

```powershell
cargo test --all-features
pwsh e2e-win\run.ps1
pwsh e2e-win\run.ps1 task
```

## Verify the Binary in a Fresh Shell

Open a new terminal after installation and verify the binary you are actually
running:

```sh
command -v mise
mise --version
```

On Windows:

```powershell
where.exe mise
mise --version
```