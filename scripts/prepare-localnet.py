#!/usr/bin/env python3
"""Set the generated scaffold config to LEZ's publishable block size."""

import json
from pathlib import Path


CONFIG_PATH = Path(".scaffold/state/sequencer_config.json")


def main() -> None:
    if not CONFIG_PATH.is_file():
        raise SystemExit(
            f"{CONFIG_PATH} is missing; run `logos-scaffold setup` first"
        )

    config = json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    config["max_block_size"] = "1 MiB"
    CONFIG_PATH.write_text(
        json.dumps(config, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"Set {CONFIG_PATH} max_block_size to 1 MiB")


if __name__ == "__main__":
    main()
