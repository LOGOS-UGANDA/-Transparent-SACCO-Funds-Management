#!/usr/bin/env python3
"""Set the generated scaffold config to LEZ's publishable block size."""

import json
from pathlib import Path


CONFIG_PATH = Path(".scaffold/state/sequencer_config.json")
WALLET_STATE_PATH = Path(".scaffold/state/wallet.state")
LOCALNET_HOME = Path(".scaffold/localnet-clean")


def main() -> None:
    if not CONFIG_PATH.is_file():
        raise SystemExit(
            f"{CONFIG_PATH} is missing; run `logos-scaffold setup` first"
        )

    config = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    config["max_block_size"] = "1 MiB"
    config["home"] = str(LOCALNET_HOME.resolve())

    if WALLET_STATE_PATH.is_file():
        wallet_state = WALLET_STATE_PATH.read_text(encoding="utf-8")
        default_address = next(
            (
                line.partition("=")[2].strip().removeprefix("Public/")
                for line in wallet_state.splitlines()
                if line.startswith("default_address=")
            ),
            None,
        )
        if default_address and not any(
            entry.get("supply_account", {}).get("account_id") == default_address
            for entry in config.get("genesis", [])
        ):
            config.setdefault("genesis", []).append(
                {
                    "supply_account": {
                        "account_id": default_address,
                        "balance": 10_000_000_000_000,
                    }
                }
            )

    CONFIG_PATH.write_text(
        json.dumps(config, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"Set {CONFIG_PATH} max_block_size to 1 MiB")


if __name__ == "__main__":
    main()
