import os
import re
import subprocess
import argparse
from multiversx_sdk import Address, AddressComputer


def get_next_test_folder():
    """Find the last test_* folder and increment it"""
    current_dir = "."
    test_folders = []

    # Find all test_* folders
    for item in os.listdir(current_dir):
        if os.path.isdir(item) and item.startswith("test_"):
            match = re.match(r"test_(\d+)", item)
            if match:
                test_folders.append(int(match.group(1)))

    if test_folders:
        # Get the highest number and add 1
        next_test_id = max(test_folders) + 1
    else:
        # No test folders found, start with test_0
        next_test_id = 0

    return next_test_id


def create_wallet_and_get_shard(folder_path, wallet_name):
    """Create a wallet using mxpy, save it to folder as PEM, and return its shard"""

    pem_file = os.path.join(folder_path, f"{wallet_name}.pem")

    result = subprocess.run(
        [
            "mxpy",
            "wallet",
            "new",
            "--format",
            "pem",
            "--outfile",
            pem_file,
        ],
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print(f"Error creating wallet {wallet_name}: {result.stderr}")
        return None, None

    try:
        with open(pem_file, "r") as f:
            first_line = f.readline().strip()
            if "-----BEGIN PRIVATE KEY for " in first_line and "-----" in first_line:
                address_str = first_line.split("for ")[1].split("-----")[0]
            else:
                print(f"Could not parse address from PEM file for {wallet_name}")
                return None, None
    except Exception as e:
        print(f"Error reading PEM file for {wallet_name}: {e}")
        return None, None

    if not address_str or not address_str.startswith("erd"):
        print(f"Could not extract valid address for {wallet_name}")
        return None, None

    try:
        address = Address.new_from_bech32(address_str)
        address_computer = AddressComputer()
        shard = address_computer.get_shard_of_address(address)
    except Exception as e:
        print(f"Error calculating shard for {wallet_name}: {e}")
        shard = 0

    return shard, address_str, pem_file


def generate_wallets_for_all_shards(folder_path, wallet_prefix):
    """Generate wallets until we have one for each shard (0, 1, 2)"""
    target_shards = {0, 1, 2}
    found_shards = {}
    attempt = 1

    print(f"  Generating {wallet_prefix} wallets for shards {target_shards}...")

    while len(found_shards) < 3:
        wallet_name = f"{wallet_prefix}_attempt_{attempt}"
        result = create_wallet_and_get_shard(folder_path, wallet_name)

        if result[0] is not None:
            shard, address, pem_file = result

            if shard in target_shards and shard not in found_shards:
                final_name = f"{wallet_prefix}_shard_{shard}.pem"
                final_path = os.path.join(folder_path, final_name)
                os.rename(pem_file, final_path)

                found_shards[shard] = {"address": address, "file": final_path}
                print(f"    ✓ Found {wallet_prefix} for shard {shard}: {address}")
            else:
                os.remove(pem_file)

        attempt += 1

        if attempt > 100:
            print(f"    Warning: Reached 100 attempts for {wallet_prefix}, stopping")
            break

    return found_shards


def generate_test_wallets(test_id):
    """Generate wallets for a single test"""
    base_path = f"test_{test_id}"
    print(f"\n=== GENERATING WALLETS FOR {base_path.upper()} ===")

    os.makedirs(base_path, exist_ok=True)

    folders = ["bridge_owners", "sovereign_owners", "bridge_services"]
    for folder in folders:
        folder_path = os.path.join(base_path, folder)
        os.makedirs(folder_path, exist_ok=True)

    print("Bridge Owners:")
    bridge_wallets = generate_wallets_for_all_shards(os.path.join(base_path, "bridge_owners"), "bridge_owner")

    print("Sovereign Owners:")
    sovereign_wallets = generate_wallets_for_all_shards(os.path.join(base_path, "sovereign_owners"), "sovereign_owner")

    print("Bridge Services:")
    service_wallets = generate_wallets_for_all_shards(os.path.join(base_path, "bridge_services"), "bridge_service")

    return {
        "test_id": test_id,
        "bridge_owners": bridge_wallets,
        "sovereign_owners": sovereign_wallets,
        "bridge_services": service_wallets,
    }


def main():
    parser = argparse.ArgumentParser(description="Generate test wallets for MultiversX sovereign chain")
    parser.add_argument("count", type=int, nargs="?", default=1, help="Number of test folders to generate (default: 1)")

    args = parser.parse_args()

    if args.count <= 0:
        print("Error: Count must be a positive integer")
        return

    try:
        subprocess.run(["mxpy", "--version"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("Error: mxpy is not installed or not in PATH")
        print("Please install mxpy: pip install multiversx-sdk-cli")
        return

    start_test_id = get_next_test_folder()
    all_results = []

    print(f"Generating {args.count} test folder(s) starting from test_{start_test_id}")

    for i in range(args.count):
        test_id = start_test_id + i
        result = generate_test_wallets(test_id)
        all_results.append(result)

    # Print final summary
    print("\n" + "=" * 80)
    print("FINAL SUMMARY")
    print("=" * 80)

    for result in all_results:
        test_id = result["test_id"]
        print(f"\ntest_{test_id}:")

        for category in ["bridge_owners", "sovereign_owners", "bridge_services"]:
            wallets = result[category]
            print(f"  {category.replace('_', ' ').title()}:")
            for shard in sorted(wallets.keys()):
                print(f"    Shard {shard}: {wallets[shard]['address']}")

    print(f"\nSuccessfully generated {len(all_results)} test folder(s)!")


if __name__ == "__main__":
    main()
