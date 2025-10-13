use crate::agent::{download_blob, list_all_blobs, login_helper};
use crate::{build_agent, MigrationError};
use bsky_sdk::api::types::string::Did;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Deserialize, Serialize)]
pub struct GetBlobRequest {
    pub did: Did,
    pub cid: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportAllBlobsRequest {
    pub origin: String,
    pub did: String,
    pub origin_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportAllBlobsResponse {
    pub successful_blobs: Vec<String>,
    pub failed_blobs: Vec<String>,
}

#[tracing::instrument]
pub async fn export_all_blobs_api(
    req: ExportAllBlobsRequest,
) -> Result<ExportAllBlobsResponse, MigrationError> {
    let agent = build_agent().await?;
    let session = login_helper(
        &agent,
        req.origin.as_str(),
        req.did.as_str(),
        req.origin_token.as_str(),
    )
    .await?;
    let blobs = list_all_blobs(&agent).await?;
    let mut path = std::env::current_dir().unwrap();
    path.push(session.did.as_str().replace(":", "-"));
    match tokio::fs::create_dir(path.as_path()).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                tracing::error!("Error creating directory: {:?}", e);
                return Err(MigrationError::Runtime {
                    message: e.to_string(),
                });
            }
        }
    }

    let mut successful_blobs = vec![];
    let mut failed_blobs = vec![];
    for blob in &blobs {
        let session = agent.get_session().await.unwrap();
        let mut filepath = std::env::current_dir().unwrap();
        filepath.push(session.did.as_str().replace(":", "-"));
        filepath.push(
            format!("{blob:?}")
                .strip_prefix("Cid(Cid(")
                .unwrap()
                .strip_suffix("))")
                .unwrap(),
        );
        if !tokio::fs::try_exists(filepath).await.unwrap() {
            let get_blob_request = GetBlobRequest {
                did: session.did.clone(),
                cid: format!("{blob:?}")
                    .strip_prefix("Cid(Cid(")
                    .unwrap()
                    .strip_suffix("))")
                    .unwrap()
                    .to_string(),
                token: session.access_jwt.clone(),
            };
            match download_blob(agent.get_endpoint().await.as_str(), &get_blob_request).await {
                Ok(mut stream) => {
                    tracing::info!("Successfully fetched missing blob");
                    let mut path = std::env::current_dir().unwrap();
                    path.push(session.did.as_str().replace(":", "-"));
                    path.push(
                        format!("{blob:?}")
                            .strip_prefix("Cid(Cid(")
                            .unwrap()
                            .strip_suffix("))")
                            .unwrap(),
                    );
                    let mut file = tokio::fs::File::create(path.as_path()).await.unwrap();

                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.unwrap();
                        file.write_all(&chunk).await.unwrap();
                    }

                    file.flush().await.unwrap();
                    successful_blobs.push(format!("{blob:?}"));
                }
                Err(e) => {
                    match e {
                        MigrationError::RateLimitReached => {
                            tracing::error!("Rate limit reached, waiting 5 minutes");
                            let five_minutes = Duration::from_secs(300);
                            tokio::time::sleep(five_minutes).await;
                        }
                        _ => {
                            //todo
                        }
                    }
                    tracing::error!("Failed to determine missing blobs");
                    failed_blobs.push(format!("{blob:?}"));
                }
            }
        }
    }

    Ok(ExportAllBlobsResponse {
        successful_blobs,
        failed_blobs,
    })
}
