#!/usr/bin/env python3
"""
Full lifecycle wrapper for interactor tests
Called by the cargo wrapper script when an interactor test is detected
"""

import os
import random
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path
from typing import List, Optional, Tuple


INTERACTOR_DIR = os.environ.get("INTERACTOR_DIR", "interactor")
INTERACTOR_PACKAGE = os.environ.get("INTERACTOR_PACKAGE_NAME", "rust-interact")
WORKSPACE_ROOT = os.environ.get("WORKSPACE_ROOT", os.getcwd())


def parse_cargo_args(args: List[str]) -> Tuple[Optional[str], Optional[str]]:
    """Parse cargo test arguments to extract test file and test name."""
    test_file = None
    test_name = None
    in_test_args = False

    i = 0
    while i < len(args):
        arg = args[i]

        if arg == "--test":
            if i + 1 < len(args):
                test_file = args[i + 1]
                i += 2
            else:
                i += 1
        elif arg == "--":
            in_test_args = True
            i += 1
        else:
            if in_test_args:
                if test_name is None and not arg.startswith("--"):
                    test_name = arg
            i += 1

    return test_file, test_name


def cleanup_orphaned_containers():
    """Clean up orphaned chain simulator containers (from previous crashed/killed tests)."""
    try:
        result = subprocess.run(["docker", "ps", "-a", "--filter", "name=chain-sim-", "--format", "{{.Names}}"], capture_output=True, text=True, check=False)

        if result.returncode != 0:
            return

        containers = [line.strip() for line in result.stdout.strip().split("\n") if line.strip()]

        for container in containers:
            if not container.startswith("chain-sim-"):
                continue

            check_result = subprocess.run(
                ["docker", "ps", "--filter", f"name=^{container}$", "--format", "{{.Names}}"], capture_output=True, text=True, check=False
            )

            if check_result.returncode == 0 and container not in check_result.stdout:
                subprocess.run(["docker", "rm", "-f", container], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
    except Exception:
        pass


def find_available_port() -> int:
    """Find an available port for the chain simulator."""
    base_port = 8085
    # Use PID, timestamp, and random component for better uniqueness in concurrent runs
    port = base_port + (os.getpid() % 1000) + (int(time.time()) % 1000) + (random.randint(0, 99))

    while port < 1024 or port > 65535:
        port = base_port + (port % 1000) + random.randint(0, 99)

    attempts = 0
    while attempts < 100:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            sock.bind(("localhost", port))
            sock.close()
        except OSError:
            port += 1
            attempts += 1
            continue

        result = subprocess.run(["docker", "ps", "--filter", f"publish={port}", "--format", "{{.ID}}"], capture_output=True, text=True, check=False)

        if result.returncode == 0 and result.stdout.strip():
            port += 1
            attempts += 1
            continue

        break

    if port > 65535:
        print("Failed to find available port after 100 attempts", file=sys.stderr)
        sys.exit(1)

    return port


def wait_for_simulator(port: int, container_name: str, max_attempts: int = 30) -> bool:
    """Wait for chain simulator to be ready."""
    import urllib.request
    import urllib.error

    url = f"http://localhost:{port}/network/config"

    for i in range(max_attempts):
        try:
            urllib.request.urlopen(url, timeout=1)
            return True
        except (urllib.error.URLError, OSError):
            if i == max_attempts - 1:
                result = subprocess.run(["docker", "logs", container_name], capture_output=True, text=True, check=False)
                if result.stdout:
                    print(result.stdout, file=sys.stderr)
                if result.stderr:
                    print(result.stderr, file=sys.stderr)
                return False
            time.sleep(1)

    return False


def cleanup_containers(container_name: str):
    """Stop and remove containers."""
    result = subprocess.run(["docker", "ps", "--format", "{{.Names}}"], capture_output=True, text=True, check=False)
    if result.returncode == 0 and container_name in result.stdout:
        print(f"Stopping chain simulator container: {container_name}", file=sys.stderr)
        subprocess.run(["docker", "stop", "-t", "10", container_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)

    result = subprocess.run(["docker", "ps", "-a", "--format", "{{.Names}}"], capture_output=True, text=True, check=False)
    if result.returncode == 0 and container_name in result.stdout:
        subprocess.run(["docker", "rm", "-f", container_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)


def cleanup_signal(signum: int, container_name: str):
    """Signal handler for cleanup."""
    cleanup_containers(container_name)
    sys.exit(128 + signum)


def main():
    """Main execution."""
    workspace_root = os.environ.get("WORKSPACE_ROOT", os.getcwd())
    script_dir = Path(__file__).parent.absolute()

    args = sys.argv[1:]
    test_file, test_name = parse_cargo_args(args)

    is_deployment_test = test_file == "always_deploy_setup_first" and test_name == "deploy_setup"

    cleanup_orphaned_containers()

    port = find_available_port()
    os.environ["CHAIN_SIMULATOR_PORT"] = str(port)

    # Use port, PID, and timestamp for unique container name (supports concurrent runs)
    container_name = f"chain-sim-{port}-{os.getpid()}-{int(time.time())}"

    signal.signal(signal.SIGINT, lambda s, f: cleanup_signal(s, container_name))
    signal.signal(signal.SIGTERM, lambda s, f: cleanup_signal(s, container_name))

    exit_code = 0
    try:
        # 1. Start chain simulator
        print(f"Starting chain simulator on port {port}...")
        result = subprocess.run(
            ["docker", "run", "-d", "-p", f"{port}:8085", "--name", container_name, "multiversx/chainsimulator"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )

        if result.returncode != 0:
            print("Failed to start chain simulator", file=sys.stderr)
            exit_code = 1
            return

        if not wait_for_simulator(port, container_name):
            print("Chain simulator failed to start after 30 seconds", file=sys.stderr)
            exit_code = 1
            return

        # 2. Remove state file
        state_file = Path(workspace_root) / INTERACTOR_DIR / "state.toml"
        if state_file.exists():
            state_file.unlink()

        # 3. Run deployment test (unless this IS the deployment test)
        if not is_deployment_test:
            original_path = os.environ.get("PATH", "")
            path_parts = original_path.split(":")
            path_parts = [p for p in path_parts if p != str(script_dir)]
            modified_path = ":".join(path_parts)

            print("Running deployment test...", file=sys.stderr)

            env = os.environ.copy()
            env["PATH"] = modified_path
            env["CHAIN_SIMULATOR_PORT"] = str(port)

            result = subprocess.run(
                [
                    "cargo",
                    "test",
                    "--package",
                    INTERACTOR_PACKAGE,
                    "--test",
                    "always_deploy_setup_first",
                    "--all-features",
                    "--",
                    "deploy_setup",
                    "--exact",
                    "--show-output",
                ],
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )

            if result.returncode != 0:
                if result.stdout:
                    print(result.stdout, file=sys.stderr)
                if result.stderr:
                    print(result.stderr, file=sys.stderr)
                print("Deployment test failed - see output above", file=sys.stderr)
                exit_code = 1
                return

            print("Deployment test succeeded.", file=sys.stderr)

        # 4. Run the actual test
        original_path = os.environ.get("PATH", "")
        path_parts = original_path.split(":")
        path_parts = [p for p in path_parts if p != str(script_dir)]
        modified_path = ":".join(path_parts)

        env = os.environ.copy()
        env["PATH"] = modified_path
        env["CHAIN_SIMULATOR_PORT"] = str(port)

        result = subprocess.run(["cargo"] + args, env=env, check=False)
        exit_code = result.returncode

    except KeyboardInterrupt:
        exit_code = 130
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        exit_code = 1
    finally:
        cleanup_containers(container_name)

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
