use crate::lib::error::DfxResult;
use anyhow::{bail, Context};
use fn_error_context::context;
use slog::info;
use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/load_assets.rs"));

#[context("Failed to load wallet wasm.")]
pub fn wallet_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_wallet_wasm) = std::env::var("DFX_WALLET_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_wallet_wasm);
        Ok(dfx_core::fs::read(dfx_wallet_wasm.as_ref())?)
    } else {
        let mut canister_assets =
            wallet_canister().context("Failed to load wallet canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read wallet canister archive entries.")?
        {
            let mut file = file.context("Failed to read wallet canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("wallet.wasm.gz")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find wallet canister archive entry.");
    }
}

#[context("Failed to load assets wasm.")]
pub fn assets_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_assets_wasm) = std::env::var("DFX_ASSETS_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_assets_wasm);
        Ok(dfx_core::fs::read(dfx_assets_wasm.as_ref())?)
    } else {
        let mut canister_assets =
            assetstorage_canister().context("Failed to load asset canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read asset canister archive entries.")?
        {
            let mut file = file.context("Failed to read asset canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("assetstorage.wasm.gz")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find asset canister archive entry.");
    }
}

#[allow(unused)]
#[context("Failed to load bitcoin wasm.")]
pub fn bitcoin_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_assets_wasm) = std::env::var("DFX_BITCOIN_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_assets_wasm);
        Ok(dfx_core::fs::read(dfx_assets_wasm.as_ref())?)
    } else {
        let mut canister_assets =
            btc_canister().context("Failed to load bitcoin canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read bitcoin canister archive entries.")?
        {
            let mut file = file.context("Failed to read bitcoin canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("ic-btc-canister.wasm.gz")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find bitcoin canister archive entry");
    }
}

pub fn management_idl() -> DfxResult<String> {
    // FIXME get idl from replica when it's available
    // Pulled from https://github.com/dfinity/interface-spec/blob/master/spec/_attachments/ic.did
    Ok(r##"
type canister_id = principal;
type wasm_module = blob;

type canister_settings = record {
    controllers : opt vec principal;
    compute_allocation : opt nat;
    memory_allocation : opt nat;
    freezing_threshold : opt nat;
    reserved_cycles_limit : opt nat;
};

type definite_canister_settings = record {
    controllers : vec principal;
    compute_allocation : nat;
    memory_allocation : nat;
    freezing_threshold : nat;
    reserved_cycles_limit : nat;
};

type change_origin = variant {
    from_user : record {
        user_id : principal;
    };
    from_canister : record {
        canister_id : principal;
        canister_version : opt nat64;
    };
};

type change_details = variant {
    creation : record {
        controllers : vec principal;
    };
    code_uninstall;
    code_deployment : record {
        mode : variant { install; reinstall; upgrade };
        module_hash : blob;
    };
    controllers_change : record {
        controllers : vec principal;
    };
};

type change = record {
    timestamp_nanos : nat64;
    canister_version : nat64;
    origin : change_origin;
    details : change_details;
};

type chunk_hash = record {
  hash : blob;
};

type http_header = record {
    name : text;
    value : text;
};

type http_request_result = record {
    status : nat;
    headers : vec http_header;
    body : blob;
};

type ecdsa_curve = variant {
    secp256k1;
};

type satoshi = nat64;

type bitcoin_network = variant {
    mainnet;
    testnet;
};

type bitcoin_address = text;

type block_hash = blob;

type outpoint = record {
    txid : blob;
    vout : nat32;
};

type utxo = record {
    outpoint : outpoint;
    value : satoshi;
    height : nat32;
};

type bitcoin_get_utxos_args = record {
    address : bitcoin_address;
    network : bitcoin_network;
    filter : opt variant {
        min_confirmations : nat32;
        page : blob;
    };
};

type bitcoin_get_utxos_query_args = record {
    address : bitcoin_address;
    network : bitcoin_network;
    filter : opt variant {
        min_confirmations : nat32;
        page : blob;
    };
};

type bitcoin_get_current_fee_percentiles_args = record {
    network : bitcoin_network;
};

type bitcoin_get_utxos_result = record {
    utxos : vec utxo;
    tip_block_hash : block_hash;
    tip_height : nat32;
    next_page : opt blob;
};

type bitcoin_get_utxos_query_result = record {
    utxos : vec utxo;
    tip_block_hash : block_hash;
    tip_height : nat32;
    next_page : opt blob;
};

type bitcoin_get_balance_args = record {
    address : bitcoin_address;
    network : bitcoin_network;
    min_confirmations : opt nat32;
};

type bitcoin_get_balance_query_args = record {
    address : bitcoin_address;
    network : bitcoin_network;
    min_confirmations : opt nat32;
};

type bitcoin_send_transaction_args = record {
    transaction : blob;
    network : bitcoin_network;
};

type millisatoshi_per_byte = nat64;

type node_metrics = record {
    node_id : principal;
    num_blocks_total : nat64;
    num_block_failures_total : nat64;
};

type create_canister_args = record {
    settings : opt canister_settings;
    sender_canister_version : opt nat64;
};

type create_canister_result = record {
    canister_id : canister_id;
};

type update_settings_args = record {
    canister_id : principal;
    settings : canister_settings;
    sender_canister_version : opt nat64;
};

type upload_chunk_args = record {
    canister_id : principal;
    chunk : blob;
};

type clear_chunk_store_args = record {
    canister_id : canister_id;
};

type stored_chunks_args = record {
    canister_id : canister_id;
};

type canister_install_mode = variant {
    install;
    reinstall;
    upgrade : opt record {
        skip_pre_upgrade : opt bool;
    };
};

type install_code_args = record {
    mode : canister_install_mode;
    canister_id : canister_id;
    wasm_module : wasm_module;
    arg : blob;
    sender_canister_version : opt nat64;
};

type install_chunked_code_args = record {
    mode : canister_install_mode;
    target_canister : canister_id;
    store_canister : opt canister_id;
    chunk_hashes_list : vec chunk_hash;
    wasm_module_hash : blob;
    arg : blob;
    sender_canister_version : opt nat64;
};

type uninstall_code_args = record {
    canister_id : canister_id;
    sender_canister_version : opt nat64;
};

type start_canister_args = record {
    canister_id : canister_id;
};

type stop_canister_args = record {
    canister_id : canister_id;
};

type canister_status_args = record {
    canister_id : canister_id;
};

type canister_status_result = record {
    status : variant { running; stopping; stopped };
    settings : definite_canister_settings;
    module_hash : opt blob;
    memory_size : nat;
    cycles : nat;
    reserved_cycles : nat;
    idle_cycles_burned_per_day : nat;
};

type canister_info_args = record {
    canister_id : canister_id;
    num_requested_changes : opt nat64;
};

type canister_info_result = record {
    total_num_changes : nat64;
    recent_changes : vec change;
    module_hash : opt blob;
    controllers : vec principal;
};

type delete_canister_args = record {
    canister_id : canister_id;
};

type deposit_cycles_args = record {
    canister_id : canister_id;
};

type http_request_args = record {
    url : text;
    max_response_bytes : opt nat64;
    method : variant { get; head; post };
    headers : vec http_header;
    body : opt blob;
    transform : opt record {
        function : func(record { response : http_request_result; context : blob }) -> (http_request_result) query;
        context : blob;
    };
};

type ecdsa_public_key_args = record {
    canister_id : opt canister_id;
    derivation_path : vec blob;
    key_id : record { curve : ecdsa_curve; name : text };
};

type ecdsa_public_key_result = record {
    public_key : blob;
    chain_code : blob;
};

type sign_with_ecdsa_args = record {
    message_hash : blob;
    derivation_path : vec blob;
    key_id : record { curve : ecdsa_curve; name : text };
};

type sign_with_ecdsa_result = record {
    signature : blob;
};

type node_metrics_history_args = record {
    subnet_id : principal;
    start_at_timestamp_nanos : nat64;
};

type node_metrics_history_result = vec record {
    timestamp_nanos : nat64;
    node_metrics : vec node_metrics;
};

type provisional_create_canister_with_cycles_args = record {
    amount : opt nat;
    settings : opt canister_settings;
    specified_id : opt canister_id;
    sender_canister_version : opt nat64;
};

type provisional_create_canister_with_cycles_result = record {
    canister_id : canister_id;
};

type provisional_top_up_canister_args = record {
    canister_id : canister_id;
    amount : nat;
};

type raw_rand_result = blob;

type stored_chunks_result = vec chunk_hash;

type upload_chunk_result = chunk_hash;

type bitcoin_get_balance_result = satoshi;

type bitcoin_get_balance_query_result = satoshi;

type bitcoin_get_current_fee_percentiles_result = vec millisatoshi_per_byte;

service ic : {
    create_canister : (create_canister_args) -> (create_canister_result);
    update_settings : (update_settings_args) -> ();
    upload_chunk : (upload_chunk_args) -> (upload_chunk_result);
    clear_chunk_store : (clear_chunk_store_args) -> ();
    stored_chunks : (stored_chunks_args) -> (stored_chunks_result);
    install_code : (install_code_args) -> ();
    install_chunked_code : (install_chunked_code_args) -> ();
    uninstall_code : (uninstall_code_args) -> ();
    start_canister : (start_canister_args) -> ();
    stop_canister : (stop_canister_args) -> ();
    canister_status : (canister_status_args) -> (canister_status_result);
    canister_info : (canister_info_args) -> (canister_info_result);
    delete_canister : (delete_canister_args) -> ();
    deposit_cycles : (deposit_cycles_args) -> ();
    raw_rand : () -> (raw_rand_result);
    http_request : (http_request_args) -> (http_request_result);

    // Threshold ECDSA signature
    ecdsa_public_key : (ecdsa_public_key_args) -> (ecdsa_public_key_result);
    sign_with_ecdsa : (sign_with_ecdsa_args) -> (sign_with_ecdsa_result);

    // bitcoin interface
    bitcoin_get_balance : (bitcoin_get_balance_args) -> (bitcoin_get_balance_result);
    bitcoin_get_balance_query : (bitcoin_get_balance_query_args) -> (bitcoin_get_balance_query_result) query;
    bitcoin_get_utxos : (bitcoin_get_utxos_args) -> (bitcoin_get_utxos_result);
    bitcoin_get_utxos_query : (bitcoin_get_utxos_query_args) -> (bitcoin_get_utxos_query_result) query;
    bitcoin_send_transaction : (bitcoin_send_transaction_args) -> ();
    bitcoin_get_current_fee_percentiles : (bitcoin_get_current_fee_percentiles_args) -> (bitcoin_get_current_fee_percentiles_result);

    // metrics interface
    node_metrics_history : (node_metrics_history_args) -> (node_metrics_history_result);

    // provisional interfaces for the pre-ledger world
    provisional_create_canister_with_cycles : (provisional_create_canister_with_cycles_args) -> (provisional_create_canister_with_cycles_result);
    provisional_top_up_canister : (provisional_top_up_canister_args) -> ();
};
"##.to_string())
}
