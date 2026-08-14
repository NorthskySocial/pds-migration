use crate::background_jobs::JobManager;
use pdsmigration_common::downloads_dir;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Periodically delete local migration artifacts (blob directories and repo
/// CAR files) that are older than `retention`.
pub async fn run_periodic_gc(jobs: JobManager, retention: Duration, interval: Duration) {
    tracing::info!(
        retention_secs = retention.as_secs(),
        interval_secs = interval.as_secs(),
        "Starting local artifact garbage collector",
    );

    loop {
        tokio::time::sleep(interval).await;

        if jobs.has_active_jobs().await {
            tracing::debug!("Skipping artifact garbage collection: jobs are still active");
            continue;
        }

        let dir = match downloads_dir() {
            Ok(dir) => dir,
            Err(error) => {
                tracing::warn!(%error, "Artifact garbage collection: cannot resolve downloads dir");
                continue;
            }
        };

        match collect_once(&dir, retention).await {
            Ok(0) => tracing::debug!("Artifact garbage collection: nothing to remove"),
            Ok(removed) => tracing::info!(removed, "Artifact garbage collection removed artifacts"),
            Err(error) => tracing::warn!(%error, "Artifact garbage collection failed"),
        }
    }
}

/// Remove expired artifacts in `dir` once, returning how many were removed.
pub async fn collect_once(dir: &Path, retention: Duration) -> std::io::Result<u64> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut removed = 0;

    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let metadata = match tokio::fs::metadata(&path).await {
            Ok(metadata) => metadata,
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Cannot stat artifact candidate");
                continue;
            }
        };

        if !is_artifact(&name, metadata.is_dir()) || !is_expired(&metadata, retention) {
            continue;
        }

        let result = if metadata.is_dir() {
            tokio::fs::remove_dir_all(&path).await
        } else {
            tokio::fs::remove_file(&path).await
        };

        match result {
            Ok(()) => {
                tracing::info!(path = %path.display(), "Removed expired local artifact");
                removed += 1;
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Failed to remove expired artifact");
            }
        }
    }

    Ok(removed)
}

/// Artifacts are named after a DID: `did-<...>` directories of blobs
/// and `did-<...>.car` repository exports.
fn is_artifact(name: &str, is_dir: bool) -> bool {
    name.starts_with("did-") && (is_dir || name.ends_with(".car"))
}

fn is_expired(metadata: &std::fs::Metadata, retention: Duration) -> bool {
    let modified = match metadata.modified() {
        Ok(modified) => modified,
        Err(_) => return false,
    };
    match SystemTime::now().duration_since(modified) {
        Ok(age) => age >= retention,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::UNIX_EPOCH;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pdsmigration-gc-{}-{}-{}",
            name,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock went backwards")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn make_artifacts(dir: &Path) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        let blobs = dir.join("did-plc-abc123");
        fs::create_dir(&blobs).expect("create blob dir");
        fs::write(blobs.join("blobcid"), b"data").expect("write blob");

        let car = dir.join("did-plc-abc123.car");
        fs::write(&car, b"car").expect("write car");

        let unrelated = dir.join("pdsmigration-web");
        fs::write(&unrelated, b"binary").expect("write binary");

        (blobs, car, unrelated)
    }

    #[test]
    fn recognizes_artifacts_only() {
        assert!(is_artifact("did-plc-abc123", true));
        assert!(is_artifact("did-plc-abc123.car", false));

        assert!(!is_artifact("did-plc-abc123.txt", false));
        assert!(!is_artifact("pdsmigration-web", true));
        assert!(!is_artifact("config.toml", false));
    }

    #[tokio::test]
    async fn removes_expired_artifacts_only() {
        let dir = temp_dir("expired");
        let (blobs, car, unrelated) = make_artifacts(&dir);

        let removed = collect_once(&dir, Duration::ZERO).await.expect("collect");

        assert_eq!(removed, 2);
        assert!(!blobs.exists());
        assert!(!car.exists());
        assert!(unrelated.exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn reports_error_for_unreadable_dir() {
        let missing = std::env::temp_dir().join("pdsmigration-gc-does-not-exist");
        assert!(collect_once(&missing, Duration::ZERO).await.is_err());
    }

    #[tokio::test]
    async fn keeps_artifacts_within_retention() {
        let dir = temp_dir("retained");
        let (blobs, car, unrelated) = make_artifacts(&dir);

        let removed = collect_once(&dir, Duration::from_secs(3_600))
            .await
            .expect("collect");

        assert_eq!(removed, 0);
        assert!(blobs.exists());
        assert!(car.exists());
        assert!(unrelated.exists());

        fs::remove_dir_all(&dir).ok();
    }
}
