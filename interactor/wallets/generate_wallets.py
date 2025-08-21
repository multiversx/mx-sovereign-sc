import os
import subprocess
from multiversx_sdk import Address, AddressComputer


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

    print(f"Generating {wallet_prefix} wallets for shards {target_shards}...")

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
                print(f"✓ Found {wallet_prefix} for shard {shard}: {address}")
            else:
                os.remove(pem_file)

        attempt += 1

        if attempt > 100:
            print(f"Warning: Reached 100 attempts for {wallet_prefix}, stopping")
            break

    return found_shards


def generate_user_wallet_for_shard_1(folder_path):
    """Generate a user wallet specifically for shard 1"""
    target_shard = 1
    attempt = 1

    print(f"Generating user wallet for shard {target_shard}...")

    while True:
        wallet_name = f"user_attempt_{attempt}"
        result = create_wallet_and_get_shard(folder_path, wallet_name)

        if result[0] is not None:
            shard, address, pem_file = result

            if shard == target_shard:
                final_name = "user.pem"
                final_path = os.path.join(folder_path, final_name)
                os.rename(pem_file, final_path)

                print(f"✓ Found user wallet for shard {shard}: {address}")
                return {"address": address, "file": final_path, "shard": shard}
            else:
                os.remove(pem_file)

        attempt += 1

        if attempt > 100:
            print(f"Warning: Reached 100 attempts for user wallet, stopping")
            break

    return None


def main():
    base_path = "test_3"

    try:
        subprocess.run(["mxpy", "--version"], capture_output=True, check=True)
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("Error: mxpy is not installed or not in PATH")
        print("Please install mxpy: pip install multiversx-sdk-cli")
        return

    os.makedirs(base_path, exist_ok=True)

    folders = ["bridge_owners", "sovereign_owners", "bridge_services"]
    for folder in folders:
        folder_path = os.path.join(base_path, folder)
        os.makedirs(folder_path, exist_ok=True)

    print("=== BRIDGE OWNERS ===")
    bridge_wallets = generate_wallets_for_all_shards(os.path.join(base_path, "bridge_owners"), "bridge_owner")

    print("\n=== SOVEREIGN OWNERS ===")
    sovereign_wallets = generate_wallets_for_all_shards(os.path.join(base_path, "sovereign_owners"), "sovereign_owner")

    print("\n=== BRIDGE SERVICES ===")
    service_wallets = generate_wallets_for_all_shards(os.path.join(base_path, "bridge_services"), "bridge_service")

    print("\n=== USER WALLET ===")
    user_wallet = generate_user_wallet_for_shard_1(base_path)

    print("\n" + "=" * 60)
    print("FINAL WALLET SUMMARY")
    print("=" * 60)

    for category, wallets in [("BRIDGE OWNERS", bridge_wallets), ("SOVEREIGN OWNERS", sovereign_wallets), ("BRIDGE SERVICES", service_wallets)]:
        print(f"\n{category}:")
        for shard in sorted(wallets.keys()):
            print(f"  Shard {shard}: {wallets[shard]['address']}")

    if user_wallet:
        print(f"\nUSER WALLET:")
        print(f"  Shard {user_wallet['shard']}: {user_wallet['address']}")

    print("\nAll wallets generated successfully!")


if __name__ == "__main__":
    main()
