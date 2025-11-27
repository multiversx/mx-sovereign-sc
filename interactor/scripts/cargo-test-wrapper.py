#!/usr/bin/env python3
"""
Full lifecycle wrapper for interactor tests
Called by the cargo wrapper script when an interactor test is detected
"""

import os
import random
import socket
import subprocess
import sys
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

    Returns:
        An available port number between 1024 and 65535.

    Raises:
        SystemExit: If no available port is found after 100 attempts.
    """
    base_port = 8085
    port = base_port + (os.getpid() % 1000) + (int(time.time()) % 1000) + random.randint(0, 99)

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


def run_parallel_tests(test_cases: List[str], args: List[str]) -> None:
    """Run multiple test cases in parallel with concurrency limit.

    Args:
        test_cases: List of test case names to run.
        args: Original cargo test arguments to reuse.

    Exits with 0 if all tests pass, 1 if any fail, 130 on KeyboardInterrupt.
    """
    max_concurrency = get_max_concurrency()
    print(f"Running {len(test_cases)} test cases with max concurrency: {max_concurrency}", file=sys.stderr)

    processes = {}
    completed_processes = set()
    exit_codes = {}
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

        if os.environ.get("GITHUB_ACTIONS") == "true":
            summary_path = os.environ.get("INTERACTOR_TEST_SUMMARY_PATH", "interactor_test_summary.md")
            try:
                with open(summary_path, "w", encoding="utf-8") as f:
                    f.write("## Interactor Test Summary\n\n")
                    f.write(f"- **Total tests**: {total_tests}\n")
                    f.write(f"- **Passed**: {passed_count}\n")
                    f.write(f"- **Failed**: {failed_count}\n\n")
                    if failed_tests:
                        f.write("### Failed tests\n")
                        for test_name in failed_tests:
                            f.write(f"- `{test_name}`\n")
            except OSError:
                pass

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
        sys.exit(130)


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
            run_parallel_tests(test_cases, args)

    port_str = os.environ.get("CHAIN_SIMULATOR_PORT")
    if port_str:
        port = int(port_str)
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
            exit_code = run_test(args, script_dir, port, test_file, test_name)
            sys.exit(exit_code)

    port = find_available_port()
    os.environ["CHAIN_SIMULATOR_PORT"] = str(port)

    random_suffix = random.randint(1000, 9999)
    container_name = f"chain-sim-{port}-{os.getpid()}-{int(time.time())}-{random_suffix}"

    exit_code = 0
    try:
        if not start_simulator_container(port, container_name):
            exit_code = 1
        exit_code = run_test(args, script_dir, port, test_file, test_name)

    except KeyboardInterrupt:
        exit_code = 130
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        exit_code = 1

    sys.exit(exit_code)


if __name__ == "__main__":
    main()
