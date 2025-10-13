use crate::MigrationError;
use bsky_sdk::api::app::bsky::actor::defs::Preferences;
use bsky_sdk::BskyAgent;
use ipld_core::ipld::Ipld;

#[tracing::instrument(skip(agent))]
pub async fn export_preferences(agent: &BskyAgent) -> Result<Preferences, MigrationError> {
    use bsky_sdk::api::app::bsky::actor::get_preferences::{Parameters, ParametersData};
    let result = agent
        .api
        .app
        .bsky
        .actor
        .get_preferences(Parameters {
            data: ParametersData {},
            extra_data: Ipld::Null,
        })
        .await
        .map_err(|error| {
            tracing::error!("Failed to export preferences: {:?}", error);
            MigrationError::Runtime {
                message: error.to_string(),
            }
        })?;
    Ok(result.preferences.clone())
}

#[tracing::instrument(skip(agent))]
pub async fn import_preferences(
    agent: &BskyAgent,
    preferences: Preferences,
) -> Result<(), MigrationError> {
    use bsky_sdk::api::app::bsky::actor::put_preferences::{Input, InputData};
    agent
        .api
        .app
        .bsky
        .actor
        .put_preferences(Input {
            data: InputData { preferences },
            extra_data: Ipld::Null,
        })
        .await
        .map_err(|error| {
            tracing::error!("Failed to import preferences: {:?}", error);
            MigrationError::Runtime {
                message: error.to_string(),
            }
        })?;
    Ok(())
}
