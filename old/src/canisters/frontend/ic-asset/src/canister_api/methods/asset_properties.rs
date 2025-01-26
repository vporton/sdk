use crate::canister_api::{
    methods::method_names::GET_ASSET_PROPERTIES,
    types::asset::{AssetDetails, AssetProperties, GetAssetPropertiesArgument},
};
use crate::error::GetAssetPropertiesError;
use crate::error::GetAssetPropertiesError::GetAssetPropertiesFailed;
use futures_intrusive::sync::SharedSemaphore;
use ic_agent::{agent::RejectResponse, AgentError};
use ic_utils::call::SyncCall;
use ic_utils::Canister;
use std::collections::HashMap;

const MAX_CONCURRENT_REQUESTS: usize = 20;

pub(crate) async fn get_assets_properties(
    canister: &Canister<'_>,
    canister_assets: &HashMap<String, AssetDetails>,
) -> Result<HashMap<String, AssetProperties>, GetAssetPropertiesError> {
    let semaphore = SharedSemaphore::new(true, MAX_CONCURRENT_REQUESTS);

    let asset_ids = canister_assets.keys().cloned().collect::<Vec<_>>();
    let futs = asset_ids
        .iter()
        .map(|asset_id| async {
            semaphore.acquire(1).await;
            get_asset_properties(canister, asset_id).await
        })
        .collect::<Vec<_>>();

    let results = futures::future::join_all(futs).await;

    let mut all_assets_properties = HashMap::new();
    for (index, result) in results.into_iter().enumerate() {
        match result {
            Ok(asset_properties) => {
                all_assets_properties.insert(asset_ids[index].to_string(), asset_properties);
            }
            // older canisters don't have get_assets_properties method
            // therefore we can break the loop
            Err(AgentError::UncertifiedReject(RejectResponse { reject_message, .. }))
                if reject_message
                    .contains(&format!("has no query method '{GET_ASSET_PROPERTIES}'"))
                    || reject_message.contains("query method does not exist") =>
            {
                break;
            }
            Err(e) => {
                return Err(GetAssetPropertiesFailed(asset_ids[index].clone(), e));
            }
        }
    }

    Ok(all_assets_properties)
}

pub(crate) async fn get_asset_properties(
    canister: &Canister<'_>,
    asset_id: &str,
) -> Result<AssetProperties, AgentError> {
    let (asset_properties,): (AssetProperties,) = canister
        .query(GET_ASSET_PROPERTIES)
        .with_arg(GetAssetPropertiesArgument(asset_id.to_string()))
        .build()
        .call()
        .await?;
    Ok(asset_properties)
}
