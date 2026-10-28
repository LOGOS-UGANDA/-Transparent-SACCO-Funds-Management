use std::{str::FromStr as _, time::Duration};

use anyhow::{bail, Context as _};
use borsh::BorshDeserialize as _;
use lee::{account::ProgramShardSelector, AccountId};
use wallet::{AccountIdentity, WalletCore};

use sacco_ledger_local_client::initialize_instruction_data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let program_id = parse_id(args.next(), "deployed program account ID")?;
    let fee_payer_id = parse_id(args.next(), "local fee payer account ID")?;
    if args.next().is_some() {
        bail!("usage: sacco-ledger-local-client <program-account-id> <local-fee-payer-account-id>");
    }

    let config_id = sacco_ledger_core::compute_config_pda(program_id);
    let wallet = WalletCore::from_env().context("load the configured local wallet")?;
    let config_account = AccountIdentity::PublicNoSign(config_id).select_program_shard(program_id);
    let tx_hash = wallet
        .send_pub_tx_paid_by(
            vec![config_account],
            initialize_instruction_data()?,
            program_id,
            Some(fee_payer_id),
        )
        .await
        .context("submit Initialize to the configured local sequencer")?;

    println!("Initialize transaction: {tx_hash}");
    println!("Config PDA: {config_id}");

    let selector = ProgramShardSelector::new(config_id, program_id);
    for _ in 0..60 {
        let account = wallet
            .get_account_view(selector)
            .await
            .context("read the config PDA from the configured local sequencer")?;
        if let Some(shard) = account.data.shards.get(&program_id) {
            let config = sacco_ledger_core::ProgramConfig::try_from_slice(shard)
                .context("decode the initialized program config")?;
            if config.version == 1 {
                println!("Initialize verified: config version {}", config.version);
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    bail!("Initialize transaction {tx_hash} was submitted, but config version 1 was not visible within 60 seconds")
}

fn parse_id(value: Option<String>, label: &str) -> anyhow::Result<AccountId> {
    let value = value.with_context(|| format!("missing {label}"))?;
    AccountId::from_str(&value).with_context(|| format!("invalid {label}"))
}
