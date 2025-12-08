# Chain Config Contract

Maintains the validator set and sovereign chain configuration. Genesis registration happens here before the bridge is opened, and later updates come through signed bridge operations.

- **Initialization:** `init` accepts an optional `SovereignConfig` (min/max validators, stakes, limits). Defaults are applied if none is provided.
- **Validator lifecycle:** Validators register/unregister during the setup phase via `register` / `unregister`. After setup, updates flow through bridge-controlled endpoints `registerBlsKey` and `unregisterBlsKey` (operation hashes are locked through the Header Verifier).
- **Configuration updates:** Owners can adjust the sovereign config during setup with `updateSovereignConfigSetupPhase`; after setup the signed path `updateSovereignConfig` is used.
- **Completing setup:** `completeSetupPhase` finalizes genesis once the minimum validator count is present. The Header Verifier then mirrors the validator set for signature checks.
- **Interactions:** The Header Verifier reads the BLS key map stored here to verify bridge bundles. Sovereign Forge deploys this contract first, and Chain Factory clones it from a template when spinning up a new sovereign.
