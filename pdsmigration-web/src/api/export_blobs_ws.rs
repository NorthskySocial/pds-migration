use super::{ExportBlobsApiRequest, ExportBlobsApiResponse};
use crate::errors::ApiError;
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::{Message, Session};
use futures_util::StreamExt;
use pdsmigration_common::{
    build_agent, download_blob, login_helper, missing_blobs, ExportBlobsRequest,
    ExportBlobsResponse, GetBlobRequest, MigrationError,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExportBlobsWsServerMsg {
    Ready,
    Started,
    Progress { blob_id: String, status: String },
    Finished { result: ExportBlobsApiResponse },
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExportBlobsWsClientMsg {
    Start { payload: ExportBlobsApiRequest },
    Close,
}

#[utoipa::path(
    get,
    path = "/export-blobs-ws",
    responses(
        (status = 101, description = "Switching Protocols to WebSocket"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Authentication error"),
        (status = 429, description = "Rate limit exceeded"),
    ),
    tag = "pdsmigration-web"
)]
#[tracing::instrument(skip(req, payload))]
#[get("/export-blobs-ws")]
pub async fn export_blobs_ws_api(
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, ApiError> {
    let (response, mut session, mut msg_stream) =
        actix_ws::handle(&req, payload).map_err(|e| ApiError::Runtime {
            message: e.to_string(),
        })?;

    // Spawn task to run the WebSocket session
    actix_rt::spawn(async move {
        if let Err(e) = session
            .text(serde_json::to_string(&ExportBlobsWsServerMsg::Ready).unwrap())
            .await
        {
            tracing::error!(error = %e, "Failed to send READY message");
            return;
        }

        // Expect the first meaningful message to be Start { payload }
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(txt) => {
                    let parsed: Result<ExportBlobsWsClientMsg, _> = serde_json::from_str(&txt);
                    match parsed {
                        Ok(ExportBlobsWsClientMsg::Start { payload }) => {
                            if let Err(e) = session
                                .text(
                                    serde_json::to_string(&ExportBlobsWsServerMsg::Started)
                                        .unwrap(),
                                )
                                .await
                            {
                                tracing::error!(error = %e, "Failed to send STARTED message");
                                // Close will be handled by dropping the session
                                break;
                            }

                            // Execute the export using the same logic as the HTTP API
                            if let Err(e) = handle_export(payload, &mut session).await {
                                tracing::error!(error = %e, "Export failed");
                                let _ = session
                                    .text(
                                        serde_json::to_string(&ExportBlobsWsServerMsg::Error {
                                            message: e,
                                        })
                                        .unwrap(),
                                    )
                                    .await;
                            }

                            // After handling one Start, we finish the handler
                            break;
                        }
                        Ok(ExportBlobsWsClientMsg::Close) => {
                            // client requested close
                            break;
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Invalid client message");
                            let _ = session
                                .text(
                                    serde_json::to_string(&ExportBlobsWsServerMsg::Error {
                                        message: "invalid message".to_string(),
                                    })
                                    .unwrap(),
                                )
                                .await;
                        }
                    }
                }
                Message::Binary(_) => {
                    // Not supported
                    let _ = session
                        .text(
                            serde_json::to_string(&ExportBlobsWsServerMsg::Error {
                                message: "binary messages not supported".to_string(),
                            })
                            .unwrap(),
                        )
                        .await;
                }
                Message::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                Message::Pong(_) => {}
                Message::Close(reason) => {
                    tracing::debug!(?reason, "Client closed");
                    break;
                }
                Message::Continuation(_) => {}
                Message::Nop => {}
            }
        }
    });

    Ok(response)
}

#[tracing::instrument(skip(session))]
async fn handle_export(
    payload: ExportBlobsApiRequest,
    session: &mut Session,
) -> Result<(), String> {
    // Map API request to common request type
    let req: ExportBlobsRequest = payload.into();
    let agent = build_agent().await?;
    login_helper(
        &agent,
        req.destination.as_str(),
        req.did.as_str(),
        req.destination_token.as_str(),
    )
    .await?;

    // Inform about progress (placeholder for future granular updates)
    let _ = session
        .text(
            serde_json::to_string(&ExportBlobsWsServerMsg::Progress {
                note: "starting export".to_string(),
            })
            .unwrap(),
        )
        .await;

    let missing_blobs = missing_blobs(&agent).await?;
    let bsky_session = login_helper(
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

    // Execute the export for each blob and report back the results
    let result: ExportBlobsResponse = export_blobs_stream(req).await.map_err(|e| e.to_string())?;

    let api_resp: ExportBlobsApiResponse = result.into();

    session
        .text(
            serde_json::to_string(&ExportBlobsWsServerMsg::Finished { result: api_resp })
                .map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tracing::instrument]
async fn determine_missing_blobs() {}

#[tracing::instrument]
pub async fn export_blobs_stream(
    req: ExportBlobsRequest,
) -> Result<ExportBlobsResponse, MigrationError> {
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
