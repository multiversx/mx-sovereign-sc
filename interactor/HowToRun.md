# Chain Simulator Interactor Workflow

This project uses the **Chain Simulator** to run tests against a **shared deployment** (“common state”) while ensuring each test also runs with its own **clean, per-test state**.

---

## Quick Start

1. **Start the Chain Simulator**

   ```bash
   sc-meta cs start
   ```

   > Keep this running while you execute tests.

2. **If you restart the simulator**, delete `state.toml` before running any test:

   ```bash
   # Linux / macOS
   find . -name state.toml -delete

   # Or manually:
   rm -f path/to/interactor/state.toml

   ```

   For a more convenient way to do the first 2 steps, a terminal alias can be set that **starts the chain simulator** and also **deletes the state.toml**. Example:
   ```
   cs() {
    rm -f interactor/state.toml
    command sc-meta cs start "$@"
   }
   ```

3. **Run the deployment test**

   Run `deploy_setup` inside `always_deploy_setup_first.rs`. This creates the **common state**.

   Examples:

   ```bash
   cargo test --package rust-interact --test always_deploy_setup_first --all-features -- deploy_setup --exact --show-output
   ```

4. **Run any tests you want**

   With the common state in place, you can run specific tests or whole suites:

   ```bash
   # Single test
   cargo test --package rust-interact --test 'file_name_without_rs' --all-features -- 'test_name' --show-output

   # All tests in a file
   cargo test --package rust-interact --test 'file_name_without_rs' --all-features --  --show-output 
   ```

---

## How It Works

- The **deployment test** (`always_deploy_setup_first.rs::test`) seeds the simulator with a **common state** (contracts deployed once).
- **Subsequent tests reuse this state**, avoiding multiple redeploys.
- Each test still runs in **isolation**:
  - A **per-test state** is created and deleted at the beginning of each new test.
  - The **common state remains** intact across all tests.

This makes tests **faster** and ensures **consistent deployments**.

---

## Troubleshooting

- **“Missing address / not deployed” error**  
  → Re-run the deployment test (`always_deploy_setup_first.rs::test`).

- **State seems stale or inconsistent**  
  → Stop the simulator, delete `state.toml`, restart it, run the deployment test, then your tests.

- **Unexpected slowness**  
  → Make sure you ran the deployment test, then only your target tests.
