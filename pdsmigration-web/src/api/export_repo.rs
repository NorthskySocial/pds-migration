use crate::errors::ApiError;
use crate::post;
use actix_web::web::Json;
use actix_web::HttpResponse;
use pdsmigration_common::ExportPDSRequest;
use std::env;

#[tracing::instrument(skip(req))]
#[post("/export-repo")]
pub async fn export_pds_api(req: Json<ExportPDSRequest>) -> Result<HttpResponse, ApiError> {
    tracing::info!("Export repository request received");
    // Download the repository file locally
    let req_inner = req.into_inner();
    let did = req_inner.did.clone();
    pdsmigration_common::export_pds_api(req_inner)
        .await
        .map_err(|e| {
            tracing::error!("Failed to export repository: {}", e);
            ApiError::Runtime {
                message: e.to_string(),
            }
        })?;

    // Upload the downloaded file to AWS S3
    let endpoint_url = env::var("ENDPOINT").map_err(|e| {
        tracing::error!("Failed to get ENDPOINT environment variable: {}", e);
        ApiError::Runtime {
            message: e.to_string(),
        }
    })?;

    tracing::debug!("Loading AWS config with endpoint: {}", endpoint_url);
    let config = aws_config::from_env()
        .region("auto")
        .endpoint_url(&endpoint_url)
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&config);

    let bucket_name = "migration".to_string();
    let file_name = did.replace(":", "-") + ".car";
    let key = "migration/".to_string() + &did.replace(":", "-") + ".car";

    tracing::debug!(
        "Uploading file {} to S3 bucket {} with key {}",
        file_name,
        bucket_name,
        key
    );

    let body = match aws_sdk_s3::primitives::ByteStream::from_path(std::path::Path::new(&file_name))
        .await
    {
        Ok(body) => {
            tracing::debug!("Successfully created ByteStream from file");
            body
        }
        Err(e) => {
            tracing::error!(
                "Failed to create ByteStream from file {}: {:?}",
                file_name,
                e
            );
            return Err(ApiError::Runtime {
                message: e.to_string(),
            });
        }
    };

    match client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(body)
        .send()
        .await
    {
        Ok(_) => {}
        Err(e) => {
            tracing::error!(
                "Failed to upload to S3: bucket={}, key={}, error={:?}",
                bucket_name,
                key,
                e
            );
            return Err(ApiError::Runtime {
                message: e.to_string(),
            });
        }
    };

    tracing::info!("Repository exported and uploaded to S3 successfully");
    Ok(HttpResponse::Ok().finish())
}
