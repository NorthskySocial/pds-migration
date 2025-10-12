use crate::agent::{download_blob, login_helper, missing_blobs};
use crate::export_all_blobs::GetBlobRequest;
use crate::{build_agent, MigrationError};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportBlobsRequest {
    pub destination: String,
    pub origin: String,
    pub did: String,
    pub origin_token: String,
    pub destination_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportBlobsResponse {
    pub successful_blobs: Vec<String>,
    pub invalid_blobs: Vec<String>,
}

#[tracing::instrument]
pub async fn export_blobs_api(
    req: ExportBlobsRequest,
) -> Result<ExportBlobsResponse, MigrationError> {
    let agent = build_agent().await?;
    login_helper(
        &agent,
        req.destination.as_str(),
        req.did.as_str(),
        req.destination_token.as_str(),
    )
    .await?;
    let missing_blobs = missing_blobs(&agent).await?;
    let session = login_helper(
        &agent,
        req.origin.as_str(),
        req.did.as_str(),
        req.origin_token.as_str(),
    )
    .await?;

    // Initialize collections to track successful and failed blob IDs
    let mut successful_blobs = Vec::new();
    let mut invalid_blobs = Vec::new();
    let mut path = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            return Err(MigrationError::Runtime {
                message: e.to_string(),
            })
        }
    };
    path.push(session.did.as_str().replace(":", "-"));
    match tokio::fs::create_dir(path.as_path()).await {
        Ok(_) => {
            tracing::info!("Successfully created directory");
        }
        Err(e) => {
            if e.kind() != ErrorKind::AlreadyExists {
                return Err(MigrationError::Runtime {
                    message: format!("{}", e),
                });
            }
        }
    }
    for missing_blob in &missing_blobs {
        tracing::debug!("Missing blob: {:?}", missing_blob);
        let session = match agent.get_session().await {
            Some(session) => session,
            None => {
                return Err(MigrationError::Runtime {
                    message: "Failed to get session".to_string(),
                });
            }
        };
        let mut filepath = match std::env::current_dir() {
            Ok(res) => res,
            Err(e) => {
                return Err(MigrationError::Runtime {
                    message: e.to_string(),
                });
            }
        };
        filepath.push(session.did.as_str().replace(":", "-"));
        filepath.push(
            missing_blob
                .record_uri
                .as_str()
                .split("/")
                .last()
                .unwrap_or("fallback"),
        );
        if !tokio::fs::try_exists(filepath).await.unwrap() {
            let missing_blob_cid = missing_blob.cid.clone();
            let blob_cid_str = format!("{missing_blob_cid:?}")
                .strip_prefix("Cid(Cid(")
                .unwrap()
                .strip_suffix("))")
                .unwrap()
                .to_string();
            let get_blob_request = GetBlobRequest {
                did: session.did.clone(),
                cid: blob_cid_str.clone(),
                token: session.access_jwt.clone(),
            };
            match download_blob(agent.get_endpoint().await.as_str(), &get_blob_request).await {
                Ok(mut stream) => {
                    tracing::info!("Successfully fetched missing blob");
                    let mut path = std::env::current_dir().unwrap();
                    path.push(session.did.as_str().replace(":", "-"));
                    path.push(&blob_cid_str);
                    let mut file = tokio::fs::File::create(path.as_path()).await.unwrap();

                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk.unwrap();
                        file.write_all(&chunk).await.unwrap();
                    }

                    file.flush().await.unwrap();
                    successful_blobs.push(blob_cid_str);
                }
                Err(e) => {
                    match e {
                        MigrationError::RateLimitReached => {
                            tracing::error!("Rate limit reached, waiting 5 minutes");
                            let five_minutes = Duration::from_secs(300);
                            tokio::time::sleep(five_minutes).await;
                        }
                        _ => {
                            tracing::error!("Failed to determine missing blobs");
                            return Err(MigrationError::Runtime {
                                message: e.to_string(),
                            });
                        }
                    }
                    tracing::error!("Failed to determine missing blobs");
                    invalid_blobs.push(blob_cid_str);
                }
            }
        }
    }
    Ok(ExportBlobsResponse {
        successful_blobs,
        invalid_blobs,
    })
}
