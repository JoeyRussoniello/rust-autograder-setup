# Installation

## Option A — Install from release (recommended)

### macOS

```bash
# 1) Download the macOS asset from the latest release
# 2) Extract and install:
tar -xzf autograder-setup-vX.Y.Z-x86_64-apple-darwin.tar.gz
sudo install -m 0755 autograder-setup-vX.Y.Z-x86_64-apple-darwin/autograder-setup /usr/local/bin/autograder-setup

# 3) Remove the Quarantine Attribute to disable macOS Gatekeeper and code signing requirement.
sudo xattr -r -d com.apple.quarantine /usr/local/bin/autograder-setup

# 4) Check that you can run it
autograder-setup --version
```

### Windows (PowerShell)

```powershell
# 1) Download the Windows .zip from the latest release
# 2) Extract and install:
Expand-Archive autograder-setup-vX.Y.Z-x86_64-pc-windows-gnu.zip -DestinationPath .

$dir = Get-ChildItem -Directory "autograder-setup-v*-x86_64-pc-windows-gnu" | Select-Object -First 1
$exe = Join-Path $dir.FullName "autograder-setup.exe"

$UserBin = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Force -Path $UserBin | Out-Null
Move-Item $exe "$UserBin\autograder-setup.exe" -Force

# Add to PATH for current session (optionally add permanently in System settings)
$env:PATH = "$UserBin;$env:PATH"
autograder-setup --version
```

## Option B — Build from source

```bash
git clone https://github.com/JoeyRussoniello/rust-autograder-setup
cd rust-autograder-setup
cargo build --release

# binary at target/release/autograder-setup.
# Add to PATH, or move the binary to the assignment repo you're configuring
```
