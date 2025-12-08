#!/usr/bin/env python3
"""
Full lifecycle wrapper for interactor tests
Called by the cargo wrapper script when an interactor test is detected
"""

import fcntl
import os
import random
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import List, Optional, Tuple


INTERACTOR_DIR = "interactor"
INTERACTOR_PACKAGE = "rust-interact"


# Maximum number of test cases to run in parallel
# Can be overridden via MAX_TEST_CONCURRENCY environment variable
def get_max_concurrency() -> int:
    """Get maximum concurrency from environment or default to 4."""
    return int(os.environ.get("MAX_TEST_CONCURRENCY", "4"))


def remove_script_dir_from_path(script_dir: Path) -> str:
    """Remove script directory from PATH to avoid wrapper recursion.

    Args:
        script_dir: Path to the script directory to remove from PATH.

    Returns:
        Modified PATH string with the script directory removed.
    """
    original_path = os.environ.get("PATH", "")
    path_parts = original_path.split(":")
    path_parts = [p for p in path_parts if p != str(script_dir)]
    return ":".join(path_parts)


def parse_cargo_args(args: List[str]) -> Tuple[Optional[str], Optional[str]]:
    """Parse cargo test arguments to extract test file and test name.

    Args:
        args: List of command-line arguments from cargo test.

    Returns:
        Tuple of (test_file, test_name) where either can be None.
        test_file is extracted from --test argument.
        test_name is the first non-flag argument after -- separator.
    """
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


def discover_test_cases(test_file: str, package: str, workspace_root: str, filter_test_name: Optional[str] = None) -> List[str]:
    """Discover all test cases in a test file using cargo test --list.

    Args:
        test_file: Name of the test file to discover cases from.
        package: Rust package name containing the tests.
        workspace_root: Root directory of the workspace.
        filter_test_name: Optional base test function name to filter by.
            If provided, only returns test cases matching this name or its rstest cases.

    Returns:
        List of discovered test case names. Empty list if discovery fails.
    """
    script_dir = Path(__file__).parent.absolute()
    env = os.environ.copy()
    env["PATH"] = remove_script_dir_from_path(script_dir)

    result = subprocess.run(
        [
            "cargo",
            "test",
            "--package",
            package,
            "--test",
            test_file,
            "--all-features",
            "--",
            "--list",
        ],
        env=env,
        cwd=workspace_root,
        capture_output=True,
        text=True,
        check=False,
    )

    if not result.stdout.strip():
        if result.stderr and "error:" in result.stderr.lower():
            print(f"Discovery error: {result.stderr}", file=sys.stderr)
        return []

    test_cases = []
    for line in result.stdout.split("\n"):
        line = line.strip()
        if line and line.endswith(": test"):
            test_name = line[:-6].strip()
            if test_name:
                if not filter_test_name or test_name == filter_test_name or test_name.startswith(f"{filter_test_name}::"):
                    test_cases.append(test_name)

    return test_cases


def find_available_port() -> int:
    """Find an available port for the chain simulator.

    Generates a port based on PID, timestamp, and random component,
    then verifies it's not in use by checking socket binding and Docker.
    Uses a lock file to prevent race conditions in parallel execution.

    Returns:
        An available port number between 1024 and 65535.

    Raises:
        SystemExit: If no available port is found after 200 attempts.
    """
    base_port = 8085
    # Use a wider range to avoid collisions in parallel execution
    port = base_port + (os.getpid() % 5000) + (int(time.time() * 1000) % 5000) + random.randint(0, 999)

    while port < 1024 or port > 65535:
        port = base_port + (port % 5000) + random.randint(0, 999)

    # Use a lock file to prevent race conditions
    lock_file_path = os.path.join(tempfile.gettempdir(), f"port_lock_{port}.lock")
    lock_file = None

    attempts = 0
    while attempts < 200:  # Increased attempts for better reliability
        # Check if port is available via socket binding
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            sock.bind(("localhost", port))
            sock.close()
        except OSError:
            port = base_port + (port % 5000) + random.randint(0, 999)
            if port < 1024 or port > 65535:
                port = base_port + (port % 5000) + random.randint(0, 999)
            attempts += 1
            continue

        # Check Docker containers using both publish filter and name pattern
        result = subprocess.run(
            ["docker", "ps", "--filter", f"publish={port}", "--format", "{{.ID}}"],
            capture_output=True,
            text=True,
            check=False,
            timeout=5,
        )

        if result.returncode == 0 and result.stdout.strip():
            port = base_port + (port % 5000) + random.randint(0, 999)
            if port < 1024 or port > 65535:
                port = base_port + (port % 5000) + random.randint(0, 999)
            attempts += 1
            continue

        # Try to acquire lock on this port
        try:
            lock_file = open(lock_file_path, "w")
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            # Double-check Docker after acquiring lock
            result = subprocess.run(
                ["docker", "ps", "--filter", f"publish={port}", "--format", "{{.ID}}"],
                capture_output=True,
                text=True,
                check=False,
                timeout=5,
            )
            if result.returncode == 0 and result.stdout.strip():
                lock_file.close()
                try:
                    os.remove(lock_file_path)
                except OSError:
                    pass
                port = base_port + (port % 5000) + random.randint(0, 999)
                if port < 1024 or port > 65535:
                    port = base_port + (port % 5000) + random.randint(0, 999)
                attempts += 1
                continue
            # Port is available, lock acquired
            break
        except (IOError, OSError):
            # Lock acquisition failed, port might be in use
            if lock_file:
                lock_file.close()
            port = base_port + (port % 5000) + random.randint(0, 999)
            if port < 1024 or port > 65535:
                port = base_port + (port % 5000) + random.randint(0, 999)
            attempts += 1
            continue

    if port > 65535 or attempts >= 200:
        if lock_file:
            lock_file.close()
            try:
                os.remove(lock_file_path)
            except OSError:
                pass
        print("Failed to find available port after 200 attempts", file=sys.stderr)
        sys.exit(1)

    # Lock will be released when process exits, but we keep it for now
    # The lock file will be cleaned up on process exit
    return port


def wait_for_simulator(port: int, container_name: str, max_attempts: int = 30) -> bool:
    """Wait for chain simulator to be ready.

    Polls the simulator's network config endpoint until it responds or max attempts reached.

    Args:
        port: Port number where the simulator is running.
        container_name: Name of the Docker container running the simulator.
        max_attempts: Maximum number of polling attempts (default: 30).

    Returns:
        True if simulator is ready, False otherwise. On failure, prints container logs.
    """
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


def filter_output(output: str) -> str:
    """Filter out duplicate and empty 'successes:' and 'failures:' sections.

    Removes redundant summary sections from cargo test output that appear
    when multiple tests run in parallel.

    Args:
        output: Raw output string from cargo test.

    Returns:
        Filtered output string with duplicate/empty sections removed.
    """
    if not output:
        return output

    lines = output.split("\n")
    filtered_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        line_stripped = line.strip()

        if line_stripped in ("successes:", "failures:"):
            next_non_empty_idx = i + 1
            while next_non_empty_idx < len(lines) and not lines[next_non_empty_idx].strip():
                next_non_empty_idx += 1

            should_skip = False
            if next_non_empty_idx < len(lines):
                next_line_stripped = lines[next_non_empty_idx].strip()
                if next_line_stripped == line_stripped:
                    should_skip = True
                elif next_line_stripped in ("successes:", "failures:") and next_line_stripped != line_stripped:
                    should_skip = True

            if should_skip:
                i = next_non_empty_idx
                continue

        filtered_lines.append(line)
        i += 1

    return "\n".join(filtered_lines)


def print_test_output(case_name: str, output: str, exit_code: int):
    """Print output for a completed test case with clear separators.

    Args:
        case_name: Name of the test case that completed.
        output: Output string from the test execution.
        exit_code: Exit code from the test (0 = success, non-zero = failure).
    """
    print(f"\n{'='*80}", file=sys.stderr)
    print(f"TEST CASE: {case_name}", file=sys.stderr)
    print(f"{'='*80}", file=sys.stderr)

    filtered_output = filter_output(output)
    if filtered_output:
        print(filtered_output, end="", file=sys.stdout)

    status = "PASSED" if exit_code == 0 else "FAILED"
    print(f"\n{'='*80}", file=sys.stderr)
    print(f"TEST CASE: {case_name} - {status} (exit code: {exit_code})", file=sys.stderr)
    print(f"{'='*80}\n", file=sys.stderr)


def run_parallel_tests(test_cases: List[str], args: List[str], workspace_root: str, test_file: Optional[str] = None) -> None:
    """Run multiple test cases in parallel with concurrency limit.

    Args:
        test_cases: List of test case names to run.
        args: Original cargo test arguments to reuse.
        workspace_root: Root directory of the workspace.
        test_file: Optional name of the test file (for summary).

    Exits with 0 if all tests pass, 1 if any fail, 130 on KeyboardInterrupt.
    """
    max_concurrency = get_max_concurrency()
    print(f"Running {len(test_cases)} test cases with max concurrency: {max_concurrency}", file=sys.stderr)

    processes = {}
    completed_processes = set()
    exit_codes = {}
    container_names = {}  # Track containers for cleanup
    test_index = 0

    try:
        while test_index < len(test_cases) and len(processes) < max_concurrency:
            case_name = test_cases[test_index]
            case_args = list(args)
            found_separator = False
            for i, arg in enumerate(case_args):
                if arg == "--":
                    if i + 1 < len(case_args) and not case_args[i + 1].startswith("--"):
                        case_args[i + 1] = case_name
                    else:
                        case_args.insert(i + 1, case_name)
                    found_separator = True
                    break
            if not found_separator:
                case_args.extend(["--", case_name])

            wrapper_script = Path(__file__).absolute()
            process = subprocess.Popen(
                [sys.executable, str(wrapper_script)] + case_args,
                env=os.environ.copy(),
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            processes[case_name] = process
            test_index += 1

        while len(completed_processes) < len(test_cases):
            for case_name, process in list(processes.items()):
                if case_name in completed_processes:
                    continue

                if process.poll() is not None:
                    output, _ = process.communicate()
                    exit_code = process.returncode
                    exit_codes[case_name] = exit_code
                    completed_processes.add(case_name)
                    del processes[case_name]

                    print_test_output(case_name, output, exit_code)

                    if test_index < len(test_cases):
                        next_case_name = test_cases[test_index]
                        case_args = list(args)
                        found_separator = False
                        for i, arg in enumerate(case_args):
                            if arg == "--":
                                if i + 1 < len(case_args) and not case_args[i + 1].startswith("--"):
                                    case_args[i + 1] = next_case_name
                                else:
                                    case_args.insert(i + 1, next_case_name)
                                found_separator = True
                                break
                        if not found_separator:
                            case_args.extend(["--", next_case_name])

                        wrapper_script = Path(__file__).absolute()
                        next_process = subprocess.Popen(
                            [sys.executable, str(wrapper_script)] + case_args,
                            env=os.environ.copy(),
                            stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT,
                            text=True,
                        )
                        processes[next_case_name] = next_process
                        test_index += 1

            if len(completed_processes) < len(test_cases):
                time.sleep(0.1)

        # Wait for all child processes to finish
        time.sleep(1)

        passed_tests = []
        failed_tests = []

        for case_name in test_cases:
            exit_code = exit_codes.get(case_name, 0)
            if exit_code == 0:
                passed_tests.append(case_name)
            else:
                failed_tests.append(case_name)

        total_tests = len(test_cases)
        passed_count = len(passed_tests)
        failed_count = len(failed_tests)

        print(f"\n{'='*80}", file=sys.stderr)
        print(f"TEST SUMMARY", file=sys.stderr)
        print(f"{'='*80}", file=sys.stderr)
        print(f"Total tests: {total_tests}", file=sys.stderr)
        print(f"Passed: {passed_count}", file=sys.stderr)
        print(f"Failed: {failed_count}", file=sys.stderr)

        if failed_tests:
            print(f"\nFailed tests:", file=sys.stderr)
            for test_name in failed_tests:
                print(f"  - {test_name}", file=sys.stderr)

        # Final cleanup at the end of the process
        # In GitHub Actions: only clean up containers from this process
        # In local: comprehensive cleanup (all containers, networks, volumes)
        cleanup_all_docker_resources()

        overall_exit_code = 0 if failed_count == 0 else 1
        sys.exit(overall_exit_code)

    except KeyboardInterrupt:
        for case_name, process in processes.items():
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        # Final cleanup at the end of the process (even on interrupt)
        # In GitHub Actions: only clean up containers from this process
        # In local: comprehensive cleanup (all containers, networks, volumes)
        cleanup_all_docker_resources()
        sys.exit(130)


def cleanup_container(container_name: str) -> None:
    """Stop and remove a Docker container.

    Args:
        container_name: Name of the container to clean up.
    """
    # Check if container exists before attempting cleanup
    check_result = subprocess.run(
        ["docker", "ps", "-a", "--filter", f"name=^{container_name}$", "--format", "{{.Names}}"],
        capture_output=True,
        text=True,
        check=False,
        timeout=5,
    )

    if check_result.returncode != 0 or not check_result.stdout.strip():
        # Container doesn't exist, nothing to clean up
        return

    try:
        subprocess.run(
            ["docker", "stop", container_name],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=10,
        )
        subprocess.run(
            ["docker", "rm", container_name],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=10,
        )
    except subprocess.TimeoutExpired:
        # Force kill if stop times out
        subprocess.run(
            ["docker", "kill", container_name],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        subprocess.run(
            ["docker", "rm", "-f", container_name],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    except Exception:
        # Ignore errors during cleanup
        pass


def is_github_actions() -> bool:
    """Check if running in GitHub Actions environment.

    Returns:
        True if running in GitHub Actions, False otherwise.
    """
    return os.environ.get("GITHUB_ACTIONS") == "true"


def cleanup_all_docker_resources(container_name: Optional[str] = None) -> None:
    """Clean up Docker resources created by tests.

    In GitHub Actions: Only cleans up the specified container (or containers matching current PID).
    In local runs: Performs comprehensive cleanup of all chain-sim- containers, networks, and volumes.

    Args:
        container_name: Optional specific container name to clean up. If None and in GitHub Actions,
            only cleans up containers matching the current process PID.
    """
    try:
        if is_github_actions():
            # In GitHub Actions: only clean up containers from this process
            if container_name:
                # Clean up specific container
                cleanup_container(container_name)
            else:
                # Clean up containers matching current PID pattern
                current_pid = os.getpid()
                result = subprocess.run(
                    ["docker", "ps", "-a", "--filter", f"name=chain-sim-", "--format", "{{.Names}}"],
                    capture_output=True,
                    text=True,
                    check=False,
                    timeout=10,
                )

                if result.returncode == 0 and result.stdout.strip():
                    for name in result.stdout.strip().split("\n"):
                        if name.strip() and f"-{current_pid}-" in name:
                            cleanup_container(name.strip())
        else:
            # Local runs: comprehensive cleanup
            # Clean up all chain-sim- containers
            result = subprocess.run(
                ["docker", "ps", "-a", "--filter", "name=chain-sim-", "--format", "{{.ID}}"],
                capture_output=True,
                text=True,
                check=False,
                timeout=10,
            )

            if result.returncode == 0 and result.stdout.strip():
                container_ids = result.stdout.strip().split("\n")
                for container_id in container_ids:
                    if container_id.strip():
                        try:
                            subprocess.run(
                                ["docker", "stop", container_id.strip()],
                                stdout=subprocess.DEVNULL,
                                stderr=subprocess.DEVNULL,
                                check=False,
                                timeout=10,
                            )
                            subprocess.run(
                                ["docker", "rm", "-f", container_id.strip()],
                                stdout=subprocess.DEVNULL,
                                stderr=subprocess.DEVNULL,
                                check=False,
                                timeout=10,
                            )
                        except Exception:
                            pass

            # Clean up dangling networks
            subprocess.run(
                ["docker", "network", "prune", "-f"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=30,
            )

            # Clean up dangling volumes
            subprocess.run(
                ["docker", "volume", "prune", "-f"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=30,
            )
    except Exception:
        # Ignore errors during cleanup
        pass


def start_simulator_container(port: int, container_name: str) -> bool:
    """Start the chain simulator Docker container and wait for it to be ready.

    Args:
        port: Port number to use for the simulator.
        container_name: Name for the Docker container.

    Returns:
        True if simulator started successfully, False otherwise.
    """
    print(f"Starting chain simulator on port {port}...", file=sys.stderr)
    result = subprocess.run(
        ["docker", "run", "-d", "-p", f"{port}:8085", "--memory=2g", "--name", container_name, "multiversx/chainsimulator"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )

    if result.returncode != 0:
        print("Failed to start chain simulator", file=sys.stderr)
        return False

    if not wait_for_simulator(port, container_name):
        print("Chain simulator failed to start after 30 seconds", file=sys.stderr)
        cleanup_container(container_name)
        return False

    return True


def run_test(args: List[str], script_dir: Path, port: int, test_file: Optional[str], test_name: Optional[str]) -> int:
    """Run the actual test case.

    Args:
        args: Original cargo test arguments.
        script_dir: Path to the script directory.
        port: Port number where the simulator is running.
        test_file: Name of the test file, if specified.
        test_name: Name of the test, if specified.

    Returns:
        Exit code from the test (0 for success, non-zero for failure).
    """
    env = os.environ.copy()
    env["PATH"] = remove_script_dir_from_path(script_dir)
    env["CHAIN_SIMULATOR_PORT"] = str(port)

    if test_file and test_name:
        test_identifier = f"{test_file}:{test_name}"
    elif test_file:
        test_identifier = f"{test_file}:*"
    else:
        test_identifier = "unknown"

    env["TEST_IDENTIFIER"] = test_identifier
    env["TEST_PORT"] = str(port)

    cargo_args = list(args)
    if "--show-output" not in cargo_args:
        found_separator = False
        for i, arg in enumerate(cargo_args):
            if arg == "--":
                found_separator = True
                cargo_args.insert(i + 1, "--show-output")
                break
        if not found_separator:
            cargo_args.extend(["--", "--show-output"])

    result = subprocess.run(["cargo"] + cargo_args, env=env, check=False)
    return result.returncode


def main():
    workspace_root = os.environ.get("WORKSPACE_ROOT", os.getcwd())
    script_dir = Path(__file__).parent.absolute()

    args = sys.argv[1:]
    test_file, test_name = parse_cargo_args(args)

    # Three cases:
    # 1. All tests: test_file specified, no test_name -> discover all tests in file
    # 2. One test with cases: test_name without "::" -> discover all cases for this test
    # 3. Specific case: test_name with "::" -> run directly, no discovery needed
    has_base_test_name = test_name and "::" not in test_name
    should_discover = test_file and (not test_name or has_base_test_name)
    filter_test_name = test_name if has_base_test_name else None

    if should_discover:
        test_cases = discover_test_cases(test_file, INTERACTOR_PACKAGE, workspace_root, filter_test_name)

        if test_cases:
            run_parallel_tests(test_cases, args, workspace_root, test_file)
        else:
            sys.exit(1)

    port_str = os.environ.get("CHAIN_SIMULATOR_PORT")
    container_name = None
    if port_str:
        # This is a child process running a specific test (spawned by parallel runner)
        # Each child creates its own container, so we need to track it for cleanup
        port = int(port_str)
        exit_code = 0

        try:
            # Check if container already exists (shouldn't happen in normal flow)
            result = subprocess.run(
                ["docker", "ps", "--filter", f"publish={port}", "--format", "{{.Names}}"],
                capture_output=True,
                text=True,
                check=False,
            )
            if result.returncode == 0 and result.stdout.strip():
                container_name = result.stdout.strip().split("\n")[0]
                print(f"Using existing chain simulator container '{container_name}' on port {port}", file=sys.stderr)
                if not wait_for_simulator(port, container_name, max_attempts=5):
                    print(f"Warning: Container {container_name} on port {port} may not be ready", file=sys.stderr)
            else:
                # Create a new container for this child process
                random_suffix = random.randint(1000, 9999)
                container_name = f"chain-sim-{port}-{os.getpid()}-{int(time.time())}-{random_suffix}"
                if not start_simulator_container(port, container_name):
                    exit_code = 1
                    container_name = None

            if exit_code == 0 and container_name:
                exit_code = run_test(args, script_dir, port, test_file, test_name)
        except KeyboardInterrupt:
            exit_code = 130
        except Exception as e:
            print(f"Unexpected error: {e}", file=sys.stderr)
            exit_code = 1
        finally:
            # Final cleanup at the end of the child process (always runs)
            if container_name:
                if is_github_actions():
                    cleanup_all_docker_resources(container_name)
                else:
                    cleanup_container(container_name)
                    # Additional comprehensive cleanup for local runs
                    cleanup_all_docker_resources()

        sys.exit(exit_code)

    # This is a single test execution (not spawned by parallel runner)
    port = find_available_port()
    os.environ["CHAIN_SIMULATOR_PORT"] = str(port)

    random_suffix = random.randint(1000, 9999)
    container_name = f"chain-sim-{port}-{os.getpid()}-{int(time.time())}-{random_suffix}"

    exit_code = 0
    try:
        if not start_simulator_container(port, container_name):
            exit_code = 1
        else:
            exit_code = run_test(args, script_dir, port, test_file, test_name)
    except KeyboardInterrupt:
        exit_code = 130
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        exit_code = 1
    finally:
        # Final cleanup at the end of the process (always runs)
        if container_name:
            if is_github_actions():
                cleanup_all_docker_resources(container_name)
            else:
                cleanup_container(container_name)
                # Additional comprehensive cleanup for local runs
                cleanup_all_docker_resources()

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
