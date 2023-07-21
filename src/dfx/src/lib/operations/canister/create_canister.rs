use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use dfx_core::canister::build_wallet_canister;
use dfx_core::identity::CallSender;
use dfx_core::network::provider::get_network_context;

use crate::lib::ledger_types::{Memo, NotifyError};
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::operations::cmc::{notify_create, transfer_cmc, MEMO_CREATE_CANISTER};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use fn_error_context::context;
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::{Agent, AgentError};
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::format;

// The cycle fee for create request is 0.1T cycles.
const CANISTER_CREATE_FEE: u128 = 100_000_000_000_u128;
// We do not know the minimum cycle balance a canister should have.
// For now create the canister with 3T cycle balance.
const CANISTER_INITIAL_CYCLE_BALANCE: u128 = 3_000_000_000_000_u128;

#[context("Failed to create canister '{}'.", canister_name)]
pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    using_icp: Option<ICPTs>,
    with_cycles: Option<u128>,
    specified_id: Option<Principal>,
    call_sender: &CallSender,
    settings: CanisterSettings,
) -> DfxResult {
    let log = env.get_logger();
    info!(log, "Creating canister {}...", canister_name);

    let config = env.get_config_or_anyhow()?;

    let mut canister_id_store = env.get_canister_id_store()?;

    let network_name = get_network_context()?;

    if let Some(remote_canister_id) = config
        .get_config()
        .get_remote_canister_id(canister_name, &network_name)
        .unwrap_or_default()
    {
        bail!(
            "{} canister is remote on network {} and has canister id: {}",
            canister_name,
            network_name,
            remote_canister_id.to_text()
        );
    }

    let non_default_network = if network_name == "local" {
        String::new()
    } else {
        format!("on network {} ", network_name)
    };

    if let Some(canister_id) = canister_id_store.find(canister_name) {
        info!(
            log,
            "{} canister was already created {}and has canister id: {}",
            canister_name,
            non_default_network,
            canister_id.to_text()
        );
        return Ok(());
    }

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let cid = match (call_sender, using_icp) {
        (CallSender::SelectedId, Some(using_icp)) => {
            create_with_ledger(env, agent, using_icp, settings).await
        }
        (CallSender::SelectedId, None) => {
            create_with_management_canister(env, agent, with_cycles, specified_id, settings).await
        }
        (CallSender::Wallet(wallet_id), None) => {
            create_with_wallet(agent, wallet_id, with_cycles, settings).await
        }
        (CallSender::Wallet(_), Some(_)) => {
            unreachable!("Cannot create canister with wallet and ICP at the same time.")
        }
    }?;
    let canister_id = cid.to_text();
    info!(
        log,
        "{} canister created {}with canister id: {}",
        canister_name,
        non_default_network,
        canister_id
    );
    canister_id_store.add(canister_name, &canister_id)?;

    Ok(())
}

async fn create_with_ledger(
    env: &dyn Environment,
    agent: &Agent,
    using_icp: ICPTs,
    settings: CanisterSettings,
) -> DfxResult<Principal> {
    let to_principal = agent.get_principal().unwrap();
    let fee = TRANSACTION_FEE;
    let memo = Memo(MEMO_CREATE_CANISTER);
    let amount = using_icp;
    let from_subaccount = None;
    let created_at_time = None;
    let subnet_type = None;
    let height = transfer_cmc(
        agent,
        memo,
        amount,
        fee,
        from_subaccount,
        to_principal,
        created_at_time,
    )
    .await?;
    println!("Using transfer at block height {height}");

    let controller = to_principal;

    let result = notify_create(agent, controller, height, subnet_type).await?;

    match result {
        Ok(principal) => {
            println!("Canister created with id: {:?}", principal.to_text());
            Ok(principal)
        }
        Err(NotifyError::Refunded {
            reason,
            block_index,
        }) => {
            match block_index {
                Some(height) => {
                    println!("Refunded at block height {height} with message: {reason}")
                }
                None => println!("Refunded with message: {reason}"),
            };
            bail!("Refunded with message: {reason}")
        }
        Err(other) => bail!("{other:?}"),
    }
}

async fn create_with_management_canister(
    env: &dyn Environment,
    agent: &Agent,
    with_cycles: Option<u128>,
    specified_id: Option<Principal>,
    settings: CanisterSettings,
) -> DfxResult<Principal> {
    let mgr = ManagementCanister::create(agent);
    let mut builder = mgr
        .create_canister()
        .as_provisional_create_with_amount(with_cycles)
        .with_effective_canister_id(env.get_effective_canister_id());
    if let Some(sid) = specified_id {
        builder = builder.as_provisional_create_with_specified_id(sid);
    }
    if let Some(controllers) = settings.controllers {
        for controller in controllers {
            builder = builder.with_controller(controller);
        }
    };
    let res = builder
        .with_optional_compute_allocation(settings.compute_allocation)
        .with_optional_memory_allocation(settings.memory_allocation)
        .with_optional_freezing_threshold(settings.freezing_threshold)
        .call_and_wait()
        .await;
    const NEEDS_WALLET: &str = "In order to create a canister on this network, you must use a wallet in order to allocate cycles to the new canister. \
                        To do this, remove the --no-wallet argument and try again. It is also possible to create a canister on this network \
                        using `dfx ledger create-canister`, but doing so will not associate the created canister with any of the canisters in your project.";
    match res {
        Ok((o,)) => Ok(o),
        Err(AgentError::HttpError(HttpErrorPayload { status, .. }))
            if (400..500).contains(&status) =>
        {
            Err(anyhow!(NEEDS_WALLET))
        }
        Err(AgentError::ReplicaError(RejectResponse {
            reject_code: RejectCode::CanisterReject,
            reject_message,
            ..
        })) if reject_message.contains("is not allowed to call ic00 method") => {
            Err(anyhow!(NEEDS_WALLET))
        }
        Err(e) => Err(e).context("Canister creation call failed."),
    }
}

async fn create_with_wallet(
    agent: &Agent,
    wallet_id: &Principal,
    with_cycles: Option<u128>,
    settings: CanisterSettings,
) -> DfxResult<Principal> {
    let wallet = build_wallet_canister(*wallet_id, agent).await?;
    let cycles = with_cycles.unwrap_or(CANISTER_CREATE_FEE + CANISTER_INITIAL_CYCLE_BALANCE);
    match wallet
        .wallet_create_canister(
            cycles,
            settings.controllers,
            settings.compute_allocation,
            settings.memory_allocation,
            settings.freezing_threshold,
        )
        .await
    {
        Ok(result) => Ok(result.canister_id),
        Err(AgentError::WalletUpgradeRequired(s)) => Err(anyhow!(
            "{}\nTo upgrade, run dfx wallet upgrade.",
            AgentError::WalletUpgradeRequired(s)
        )),
        Err(other) => Err(anyhow!(other)),
    }
}
