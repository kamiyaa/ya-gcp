use crate::{
    builder, grpc,
    pubsub::{api, PublisherClient, SubscriberClient},
};

// Pubsub's maximum message size is 10MB, larger than tonic's default of 4MB
// (https://github.com/GoogleCloudPlatform/pubsub/issues/164)
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

// re-export traits and types necessary for the bounds on public functions
#[allow(unreachable_pub)] // the reachability lint seems faulty with parent module re-exports
pub use http::Uri;
#[allow(unreachable_pub)]
pub use tower::make::MakeConnection;

config_default! {
    /// Configuration for connecting to pubsub
    #[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
    pub struct PubSubConfig {
        /// Endpoint to connect to pubsub over.
        @default("https://pubsub.googleapis.com/v1".into(), "PubSubConfig::default_endpoint")
        pub endpoint: String,

        /// The authorization scopes to use when requesting auth tokens
        @default(vec!["https://www.googleapis.com/auth/pubsub".into()], "PubSubConfig::default_auth_scopes")
        pub auth_scopes: Vec<String>,
    }
}

/// An error encountered when building PubSub clients
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BuildError(#[from] tonic::transport::Error);

impl builder::ClientBuilder {
    async fn pubsub_authed_service(
        &self,
        config: PubSubConfig,
    ) -> Result<grpc::DefaultGrpcImpl, BuildError> {
        let connection = tonic::transport::Endpoint::new(config.endpoint)?
            .connect()
            .await?;

        Ok(grpc::DefaultGrpcImpl::new(
            connection,
            self.auth.clone(),
            config.auth_scopes,
        ))
    }

    /// Create a client for publishing to the pubsub service
    pub async fn build_pubsub_publisher(
        &self,
        config: PubSubConfig,
    ) -> Result<PublisherClient, BuildError> {
        // the crate's client will wrap the raw grpc client to add features/functions/ergonomics
        Ok(PublisherClient::from_raw_api(
            api::publisher_client::PublisherClient::new(self.pubsub_authed_service(config).await?)
                .max_decoding_message_size(MAX_MESSAGE_SIZE),
        ))
    }

    /// Create a client for subscribing to the pubsub service
    pub async fn build_pubsub_subscriber(
        &self,
        config: PubSubConfig,
    ) -> Result<SubscriberClient, BuildError> {
        // the crate's client will wrap the raw grpc client to add features/functions/ergonomics
        Ok(SubscriberClient::from_raw_api(
            api::subscriber_client::SubscriberClient::new(
                self.pubsub_authed_service(config).await?,
            )
            .max_decoding_message_size(MAX_MESSAGE_SIZE),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn config_default() {
        let config = PubSubConfig::default();
        assert_eq!(config.endpoint, "https://pubsub.googleapis.com/v1");
    }
}
