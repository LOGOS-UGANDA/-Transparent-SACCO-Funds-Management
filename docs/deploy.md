# Phase 2 deployment runbook

Phase 2 starts with local development. The SACCO program records off-chain
deposits, loan activity, and expenses; it does not receive, hold, or transfer
SACCO money. This runbook does not require a public testnet wallet, token
top-up, or fee payment.

## Current status

- `scaffold.toml` pins the project runtime to LEZ `v0.3.1`, SPEL
  `v0.7.0-rc.3`, and circuits `0.5.7`. SPEL is a separate setup tool and has
  its own LEZ dependency; the local Initialize client and program use LEZ
  `v0.3.1`.
- The repository contains a small local client that sends `Initialize` with
  the pinned LEZ v0.3.1 wallet API and confirms the resulting config state.
  `logosctl call` invokes Logos Core modules; it does not call an LEZ guest
  program.
- Program account IDs are deployment outputs and can change when the guest
  binary changes. Record the full ID from each deployment alongside the guest
  commit; do not treat an old ID as canonical.
- Scaffold CLI 0.4.0 and 0.4.1 generate an 8 MiB sequencer block limit, while
  the pinned LEZ accepts at most 1.75 MiB. The checked-in preparation script
  changes the ignored local sequencer config to 1 MiB after setup, points it at
  `.scaffold/localnet-clean`, and seeds the configured local fee payer with
  valueless genesis test units.
- On scaffold CLI 0.4.0, `localnet start` rewrites the prepared config with its
  8 MiB default immediately before spawning. Start the setup-built sequencer
  binary directly with the prepared config. The `logos-scaffold deploy` wrapper
  also calls a wallet subcommand unavailable in 0.4.0; use the working wallet
  `program-loader deploy` command below.

## 1. Build the local tooling and guest

Run from the repository root. `setup` syncs the configured LEZ, SPEL, and
circuits versions and builds the local sequencer, wallet, and `spel` binaries.
If no prebuilt sequencer exists for the pin, it falls back to a source build.
This repository has no root `Cargo.toml`, so build its workspace by manifest.
The pinned RocksDB build needs libclang; install `libclang-dev` (and clang) or
set `LIBCLANG_PATH` to an existing `libclang.so` directory.

```bash
RUSTUP_TOOLCHAIN=1.98.1 LIBCLANG_PATH=/path/to/libclang BINDGEN_EXTRA_CLANG_ARGS='-I/usr/lib/gcc/x86_64-linux-gnu/14/include -I/usr/include' logos-scaffold setup --prebuilt
CARGO_NET_OFFLINE=true LIBCLANG_PATH=/path/to/libclang BINDGEN_EXTRA_CLANG_ARGS='-I/usr/lib/gcc/x86_64-linux-gnu/14/include -I/usr/include' cargo +1.98.1 build --release --manifest-path programs/Cargo.toml
RUSTFLAGS='--cfg getrandom_backend="custom"' cargo +risc0 build --release --target riscv32im-risc0-zkvm-elf --manifest-path programs/sacco_ledger/methods/guest/Cargo.toml
python3 scripts/prepare-localnet.py
```

After stopping any local sequencer, clear its ignored data directory before a
clean run, then run `prepare-localnet.py` after the final `setup` command. The
script changes only ignored local state. Do not run `logos-scaffold build`
after preparing: it reruns setup, and this repository also has no root Cargo
manifest. Local compilation and localnet transactions do not submit anything
to a public network or require real tokens.

Start the sequencer binary built by setup. With port 3040 and metrics port 9000
free, run:

```bash
SEQUENCER_BIN="$(find "${XDG_CACHE_HOME:-$HOME/.cache}/logos-scaffold/repos/lez/1a003bfbfb89fcf987f62975cd08a401169e1b8c/target/release" -maxdepth 1 -name sequencer_service -type f -print -quit)"
RISC0_DEV_MODE=1 RUST_LOG=info "$SEQUENCER_BIN" .scaffold/state/sequencer_config.json --listen-address 127.0.0.1 --port 3040 --metrics-address 127.0.0.1:9000
```

If either port is occupied, choose unused RPC and metrics ports and update
`.scaffold/wallet/wallet_config.json` `sequencer.sequencer_addr` to the RPC
port before running wallet commands. Check connectivity with
`logos-scaffold wallet -- check-health`.

## 2. Deploy to the local sequencer

```bash
logos-scaffold wallet list --long
logos-scaffold wallet -- program-loader deploy \
  --elf programs/target/riscv-guest/sacco_ledger-methods/sacco_ledger-guest/riscv32im-risc0-zkvm-elf/release/sacco_ledger.bin \
  --segments '<segment-1>' '<segment-2>' '<segment-3>' '<segment-4>' \
  --header '<empty-header-account>' --payer '<funded-local-account>'
```

Use the empty header and segment accounts shown by `wallet list --long`, plus
the fee payer funded in local genesis. Capture the full `Header account id`
printed on success; this is the deployed program account used by the call
below and can differ between guest builds.

## 3. Submit a ledger instruction

Choose the local fee payer from `logos-scaffold wallet list --long`; setup
seeds local accounts for the sequencer's genesis state. Use the deployed
program account ID and submit the call with the repository's local client:

```bash
export NSSA_WALLET_HOME_DIR="$PWD/.scaffold/wallet"
export LEE_WALLET_HOME_DIR="$PWD/.scaffold/wallet"
PROGRAM_ACCOUNT_ID='<header account id from program-loader deploy>'
LOCAL_FEE_PAYER_ID='<seeded local account ID>'
CARGO_NET_OFFLINE=true cargo +1.98.1 run --release \
  --manifest-path programs/Cargo.toml -p sacco_ledger_local_client -- \
  "$PROGRAM_ACCOUNT_ID" "$LOCAL_FEE_PAYER_ID"
```

The client prints the config PDA and transaction hash, then exits successfully
only after it reads `ProgramConfig.version = 1` from the local sequencer. This
initializes only the program registry; it does not record or transfer SACCO
funds.

## 4. Public testnet

Deferred. Public testnet deployment and calls are not part of this local
workflow or this preparation PR. Do not top up a wallet or submit public
transactions as part of this runbook.

## Recorded local Initialize run

On the clean LEZ `v0.3.1` / circuits `0.5.7` setup, scaffold 0.4.0 generated
the sequencer and wallet binaries. `logos-scaffold localnet start` reset the
prepared block limit to 8 MiB, so the setup-built sequencer was started directly
with the prepared config. Another local E2E job occupied 3040 and 9000; this run
used RPC port 3041 and metrics port 9001, with the local wallet endpoint set to
`http://127.0.0.1:3041`.

The successful build commands used the available libclang at
`/tmp/lez-libclang/clang/native`:

```bash
RUSTUP_TOOLCHAIN=1.98.1 LIBCLANG_PATH=/tmp/lez-libclang/clang/native BINDGEN_EXTRA_CLANG_ARGS='-I/usr/lib/gcc/x86_64-linux-gnu/14/include -I/usr/include' logos-scaffold setup --prebuilt
CARGO_NET_OFFLINE=true LIBCLANG_PATH=/tmp/lez-libclang/clang/native BINDGEN_EXTRA_CLANG_ARGS='-I/usr/lib/gcc/x86_64-linux-gnu/14/include -I/usr/include' cargo +1.98.1 build --release --manifest-path programs/Cargo.toml
RUSTFLAGS='--cfg getrandom_backend="custom"' cargo +risc0 build --release --target riscv32im-risc0-zkvm-elf --manifest-path programs/sacco_ledger/methods/guest/Cargo.toml
```

After setup, the clean-state preparation and port configuration were:

```bash
python3 -c 'import shutil; shutil.rmtree(".scaffold/localnet-clean", ignore_errors=True)'
python3 scripts/prepare-localnet.py
python3 -c 'import json; from pathlib import Path; p=Path(".scaffold/state/sequencer_config.json"); c=json.loads(p.read_text()); c["port"]=3041; c["metrics_address"]="127.0.0.1:9001"; p.write_text(json.dumps(c,indent=2)+"\n"); p=Path(".scaffold/wallet/wallet_config.json"); w=json.loads(p.read_text()); w["sequencer"]["sequencer_addr"]="http://127.0.0.1:3041"; p.write_text(json.dumps(w,indent=2)+"\n")'
```

The sequencer start command was:

```bash
SEQUENCER_BIN="$(find "${XDG_CACHE_HOME:-$HOME/.cache}/logos-scaffold/repos/lez/1a003bfbfb89fcf987f62975cd08a401169e1b8c/target/release" -maxdepth 1 -name sequencer_service -type f -print -quit)"
RISC0_DEV_MODE=1 RUST_LOG=info "$SEQUENCER_BIN" .scaffold/state/sequencer_config.json --port 3041 --listen-address 127.0.0.1 --metrics-address 127.0.0.1:9001
```

The exact successful deployment command was:

```bash
logos-scaffold wallet -- program-loader deploy \
  --elf programs/target/riscv-guest/sacco_ledger-methods/sacco_ledger-guest/riscv32im-risc0-zkvm-elf/release/sacco_ledger.bin \
  --segments Public/4QmPjSrPsJu7LDVTE5VyJ1dzukcEot9ypZJsbMXULFEU Public/GU4JEfTSfVKYrGGw8p1WqtSpqxaZ98ettmib61irRU6P Public/CoppkKeV5Qy65572KtzEVWYjUeTu9BhoSdAUhRouftRN Public/BbJLQ5LqNXH7piaw2cBuLaodfVmtwenXfrBQ9S6EziAh \
  --header Public/Ge4zuh3t2Kkg9y59YR9HCmaKwrqX18DvJSV7XRwg55eQ \
  --payer Public/6eWvrckssUpTjf8iEMiq8yyQ5AJ2LM2dCWhvGLsLap9F
```

The successful client command (with both wallet-home variables exported as
above) was:

```bash
NSSA_WALLET_HOME_DIR="$PWD/.scaffold/wallet" LEE_WALLET_HOME_DIR="$PWD/.scaffold/wallet" CARGO_NET_OFFLINE=true RUSTUP_TOOLCHAIN=1.98.1 LIBCLANG_PATH=/tmp/lez-libclang/clang/native BINDGEN_EXTRA_CLANG_ARGS='-I/usr/lib/gcc/x86_64-linux-gnu/14/include -I/usr/include' cargo +1.98.1 run --release \
  --manifest-path programs/Cargo.toml -p sacco_ledger_local_client -- \
  Ge4zuh3t2Kkg9y59YR9HCmaKwrqX18DvJSV7XRwg55eQ \
  6eWvrckssUpTjf8iEMiq8yyQ5AJ2LM2dCWhvGLsLap9F
```

Output:

```text
Initialize transaction: 074afc038e148a98213f2b6f01a1e04ca38914045baeb0b51ad160ebaaa246fc
Config PDA: GHmW3j89uVbH3NnLzGNUEzfuPLLCa1cBWZdRpczq6vXm
Initialize verified: config version 1
```

The deployed header/program account ID was
`Ge4zuh3t2Kkg9y59YR9HCmaKwrqX18DvJSV7XRwg55eQ`. The Initialize call changed
only the local program config; it did not record or transfer SACCO funds.

## Related documentation

- [Program instructions, local deployment, and known gaps](../programs/sacco_ledger/README.md)
- [Attestation architecture](./architecture.md)
- [Testing guide](./testing.md)
