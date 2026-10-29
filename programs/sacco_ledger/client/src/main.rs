use std::{str::FromStr as _, time::Duration};

use anyhow::{bail, Context as _};
use borsh::BorshDeserialize as _;
use lee::{AccountId, ProgramShardSelector};
use wallet::{AccountIdentity, AccountMention, WalletCore};

use sacco_ledger_local_client::{initialize_instruction_data, instruction_data_from_json};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    if command == "call" {
        let program_id = parse_id(args.next(), "deployed program account ID")?;
        let fee_payer_id = parse_id(args.next(), "local fee payer account ID")?;
        let accounts_json = args.next().context("missing account list JSON")?;
        let instruction_json = args.next().context("missing instruction JSON")?;
        if args.next().is_some() {
            bail!("usage: sacco-ledger-local-client call <program-id> <fee-payer-id> <accounts-json> <instruction-json>");
        }
        let account_specs: Vec<AccountSpec> =
            serde_json::from_str(&accounts_json).context("parse account list JSON")?;
        let accounts = account_specs
            .into_iter()
            .map(|spec| {
                let id = AccountId::from_str(&spec.id).context("invalid account ID")?;
                let identity = if spec.signer {
                    AccountIdentity::Public(id)
                } else {
                    AccountIdentity::PublicNoSign(id)
                };
                Ok(identity.select_program_shard(program_id))
            })
            .collect::<anyhow::Result<Vec<AccountMention>>>()?;
        let data = instruction_data_from_json(&instruction_json)
            .context("parse and serialize sacco_ledger instruction JSON")?;
        let wallet = WalletCore::from_env().context("load the configured local wallet")?;
        let tx_hash = wallet
            .send_pub_tx_paid_by(accounts, data, program_id, Some(fee_payer_id))
            .await
            .context("submit instruction to the configured LEZ sequencer")?;
        println!("Transaction: {tx_hash}");
        return Ok(());
    }
    if command == "read" {
        let account_type = args.next().context("missing account type")?;
        let program_id = parse_id(args.next(), "deployed program account ID")?;
        let account_id = parse_id(args.next(), "account ID")?;
        if args.next().is_some() {
            bail!("usage: sacco-ledger-local-client read <type> <program-id> <account-id>");
        }
        let wallet = WalletCore::from_env().context("load the configured local wallet")?;
        let account = wallet
            .get_account_view(ProgramShardSelector::new(account_id, program_id))
            .await
            .context("read account from the configured LEZ sequencer")?;
        let Some(shard) = account.data.shards.get(&program_id) else {
            bail!("account has no sacco_ledger shard");
        };
        let bytes = shard.as_ref();
        let value = match account_type.as_str() {
            "sacco" => serde_json::to_value(sacco_ledger_core::Sacco::try_from_slice(bytes)?)?,
            "member" => serde_json::to_value(sacco_ledger_core::Member::try_from_slice(bytes)?)?,
            "deposit" => serde_json::to_value(sacco_ledger_core::Deposit::try_from_slice(bytes)?)?,
            "loan" => serde_json::to_value(sacco_ledger_core::Loan::try_from_slice(bytes)?)?,
            "expense" => serde_json::to_value(sacco_ledger_core::Expense::try_from_slice(bytes)?)?,
            "summary" => {
                serde_json::to_value(sacco_ledger_core::PeriodSummary::try_from_slice(bytes)?)?
            }
            "config" => {
                serde_json::to_value(sacco_ledger_core::ProgramConfig::try_from_slice(bytes)?)?
            }
            _ => bail!(
                "unknown account type; expected sacco|member|deposit|loan|expense|summary|config"
            ),
        };
        println!("{}", serde_json::to_string(&value)?);
        return Ok(());
    }
    // Preserve the original two-argument Initialize workflow for issue #4.
    let program_arg = if command == "initialize" {
        args.next()
    } else {
        Some(command)
    };
    let program_id = parse_id(program_arg, "deployed program account ID")?;
    let fee_payer_id = parse_id(args.next(), "local fee payer account ID")?;
    if args.next().is_some() {
        bail!("usage: sacco-ledger-local-client [initialize] <program-account-id> <local-fee-payer-account-id>");
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

#[derive(serde::Deserialize)]
struct AccountSpec {
    id: String,
    signer: bool,
}

fn parse_id(value: Option<String>, label: &str) -> anyhow::Result<AccountId> {
    let value = value.with_context(|| format!("missing {label}"))?;
    AccountId::from_str(&value).with_context(|| format!("invalid {label}"))
}
