# Automatic Test Wrapper

This directory contains scripts that automatically handle the full lifecycle for running interactor tests.

## What It Does

When you run `cargo test` for interactor tests, the wrapper automatically:
1. Starts a chain simulator Docker container on a unique port
2. Runs the deployment test to set up common state
3. Deletes `state.toml` for clean test isolation
4. Runs your actual test with the correct port configuration
5. Cleans up the Docker container on exit (success or failure)

## Setup (One-Time)

### Step 1: Install direnv

**macOS:**
```bash
brew install direnv
```

**Linux:**
```bash
sudo apt install direnv  # or use your package manager
```

### Step 2: Add direnv to your shell

**For bash** - add to `~/.bashrc`:
```bash
eval "$(direnv hook bash)"
```

**For zsh** - add to `~/.zshrc`:
```zsh
eval "$(direnv hook zsh)"
```

Then reload your shell:
```bash
source ~/.bashrc  # or source ~/.zshrc
```

### Step 3: Allow direnv in this repo

```bash
cd /path/to/mx-sovereign-sc
direnv allow
```

**Any changes** made to the **`.envrc`** file require running `direnv allow` again for the changes to take effect.

**Note for IDEs**: Some IDEs (like VS Code) may run commands without direnv active. If tests fail with "Connection refused" on port 8085, the wrapper isn't active. See "IDE Configuration" section below.

## How It Works

The `.envrc` file automatically adds `interactor/scripts/` to PATH when you enter the directory. The `cargo` wrapper script in that directory intercepts `cargo test` commands:

1. **You run `cargo test`** → Shell finds our wrapper (it's first in PATH)
2. **Wrapper checks** if it's an interactor test (package name, test file path, or current directory)
3. **If it's an interactor test:**
   - Routes to `cargo-test-wrapper.py` which handles the full lifecycle:
     - Start chain simulator on a unique port
     - Remove `state.toml`
     - Run the deployment test (unless this IS the deployment test)
     - Run the actual test
     - Stop chain simulator
4. **Non-interactor tests** pass through to the real cargo command

## IDE Configuration

Some IDEs don't inherit the direnv environment. If you see "Connection refused" on port 8085, the wrapper isn't active.

**Option 1: Run from terminal** (Recommended)
Just run tests from your terminal where direnv is active:
```bash
cargo test --package rust-interact --test complete_flow_tests --all-features -- test_name
```

**Option 2: VS Code Configuration**

To use the "Run Test" buttons in VS Code, add the following to `.vscode/settings.json`:

```json
{ 
  // Set PATH so rust-analyzer can find the wrapper
  "rust-analyzer.server.extraEnv": {
    "PATH": "${workspaceFolder}/interactor/scripts:${env:PATH}",
    "WORKSPACE_ROOT": "${workspaceFolder}"
  },
  
  // Set PATH for integrated terminal (so manual cargo commands use wrapper)
  "terminal.integrated.env.linux": {
    "PATH": "${workspaceFolder}/interactor/scripts:${env:PATH}"
  }
}
```

**After configuring**, you need to:
  - **Reload VS Code window**: Press `Ctrl+Shift+P` → "Developer: Reload Window" (Or restart VS Code completely)


**If it still doesn't work**: The test runner might not be using the wrapper. In that case, use Option 1 (terminal) or Option 3 (direct wrapper call).

**Option 3: Use the wrapper directly**
You can also call the wrapper script directly:
```bash
./interactor/scripts/cargo-test-wrapper.sh test --package rust-interact --test complete_flow_tests --all-features -- test_name
```
