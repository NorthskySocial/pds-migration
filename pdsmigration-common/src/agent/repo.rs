use crate::{GetRepoRequest, MigrationError};
use bsky_sdk::api::types::string::Did;
use bsky_sdk::BskyAgent;
use ipld_core::ipld::Ipld;

#[tracing::instrument]
pub async fn download_repo(
    pds_host: &str,
    request: &GetRepoRequest,
) -> Result<impl futures_core::Stream<Item = Result<bytes::Bytes, reqwest::Error>>, MigrationError>
{
    let client = reqwest::Client::new();

    let url = format!("{pds_host}/xrpc/com.atproto.sync.getRepo");
    let result = client
        .get(url)
        .query(&[("did", request.did.as_str().to_string())])
        .header("Content-Type", "application/json")
        .bearer_auth(request.token.clone())
        .send()
        .await;
    match result {
        Ok(output) => {
            let ratelimit_remaining = match output.headers().get("ratelimit-remaining") {
                None => 1000,
                Some(rate_limit_remaining) => rate_limit_remaining
                    .to_str()
                    .unwrap_or("1000")
                    .parse::<i32>()
                    .unwrap_or(1000),
            };
            if ratelimit_remaining < 100 {
                tracing::error!("Ratelimit reached");
                return Err(MigrationError::RateLimitReached);
            }

            match output.status() {
                reqwest::StatusCode::OK => {
                    tracing::info!("Started downloading Repo");
                    Ok(output.bytes_stream())
                }
                _ => {
                    tracing::error!("Runtime Error downloading Repo: {:?}", output);
                    Err(MigrationError::Upstream {
                        message: "Runtime Error downloading Repo".to_string(),
                    })
                }
            }
        }
        Err(e) => {
            tracing::error!("Unexpected Error downloading Repo: {:?}", e);
            Err(MigrationError::Runtime {
                message: "Unexpected Error downloading Repo".to_string(),
            })
        }
    }
}

#[tracing::instrument(skip(agent))]
pub async fn account_import(agent: &BskyAgent, filepath: &str) -> Result<(), MigrationError> {
    agent
        .api
        .com
        .atproto
        .repo
        .import_repo(tokio::fs::read(filepath).await.unwrap())
        .await
        .map_err(|error| {
            tracing::error!("Failed to import account: {:?}", error);
            MigrationError::Runtime {
                message: error.to_string(),
            }
        })?;
    Ok(())
}

#[tracing::instrument(skip(agent))]
pub async fn account_export(agent: &BskyAgent, did: &Did) -> Result<(), MigrationError> {
    use bsky_sdk::api::com::atproto::sync::get_repo::{Parameters, ParametersData};
    let result = agent
        .api
        .com
        .atproto
        .sync
        .get_repo(Parameters {
            data: ParametersData {
                did: did.clone(),
                since: None,
            },
            extra_data: Ipld::Null,
        })
        .await;
    match result {
        Ok(output) => {
            tokio::fs::write(did.as_str().to_string().replace(":", "-") + ".car", output)
                .await
                .map_err(|error| {
                    tracing::error!("Failed write repo bytes to file: {:?}", error);
                    MigrationError::Runtime {
                        message: error.to_string(),
                    }
                })?;
            tracing::info!("write success");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to export account: {:?}", e);
            Err(MigrationError::Upstream {
                message: e.to_string(),
            })
        }
    }
}
