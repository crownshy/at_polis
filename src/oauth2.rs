use atrium_api::agent::Agent;
use atrium_identity::{
    did::{CommonDidResolver, CommonDidResolverConfig, DEFAULT_PLC_DIRECTORY_URL},
    handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig, DnsTxtResolver},
};
use atrium_oauth::{
    store::{session::MemorySessionStore, state::MemoryStateStore},
    AtprotoClientMetadata, AtprotoLocalhostClientMetadata, AuthorizeOptions, DefaultHttpClient,
    KnownScope, OAuthClient, OAuthClientConfig, OAuthResolverConfig, OAuthSession, Scope,
};
use atrium_xrpc::http::Uri;
use hickory_resolver::{
    config::ResolverConfig, name_server::TokioConnectionProvider, Resolver, TokioResolver,
};
use std::{error::Error, sync::Arc};

// Type aliases for the configured OAuth types
pub type ConfiguredOAuthClient = OAuthClient<
    MemoryStateStore,
    MemorySessionStore,
    CommonDidResolver<DefaultHttpClient>,
    AtprotoHandleResolver<HickoryDnsTxtResolver, DefaultHttpClient>,
    DefaultHttpClient,
>;

pub type ConfiguredOAuthSession = OAuthSession<
    DefaultHttpClient,
    CommonDidResolver<DefaultHttpClient>,
    AtprotoHandleResolver<HickoryDnsTxtResolver, DefaultHttpClient>,
    MemorySessionStore,
>;

pub struct HickoryDnsTxtResolver {
    resolver: TokioResolver,
}

impl Default for HickoryDnsTxtResolver {
    fn default() -> Self {
        let resolver = Resolver::builder_with_config(
            ResolverConfig::default(),
            TokioConnectionProvider::default(),
        )
        .build();
        Self { resolver }
    }
}

impl DnsTxtResolver for HickoryDnsTxtResolver {
    async fn resolve(
        &self,
        query: &str,
    ) -> core::result::Result<Vec<String>, Box<dyn Error + Send + Sync + 'static>> {
        Ok(self
            .resolver
            .txt_lookup(query)
            .await?
            .iter()
            .map(|txt| txt.to_string())
            .collect())
    }
}

pub async fn create_oauth_client() -> Result<ConfiguredOAuthClient, anyhow::Error> {
    let http_client = Arc::new(DefaultHttpClient::default());

    let resolver = OAuthResolverConfig {
        did_resolver: CommonDidResolver::new(CommonDidResolverConfig {
            plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
            http_client: Arc::clone(&http_client),
        }),
        handle_resolver: AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
            dns_txt_resolver: HickoryDnsTxtResolver::default(),
            http_client: Arc::clone(&http_client),
        }),
        authorization_server_metadata: Default::default(),
        protected_resource_metadata: Default::default(),
    };

    let config = OAuthClientConfig {
        client_metadata: AtprotoLocalhostClientMetadata {
            redirect_uris: Some(vec![String::from(
                "http://127.0.0.1:5173/api/oauth/callback",
            )]),
            scopes: Some(vec![
                Scope::Known(KnownScope::Atproto),
                Scope::Known(KnownScope::TransitionGeneric),
                Scope::Unknown("repo:scot.comhairle.testingPolisPollV1?action=create".into()),
            ]),
        },
        keys: None,
        resolver,
        state_store: MemoryStateStore::default(),
        session_store: MemorySessionStore::default(),
    };

    let client = OAuthClient::new(config)?;
    Ok(client)
}
// Click the URL and sign in,
// then copy and paste the URL like "http://127.0.0.1/callback?iss=...&code=..." after it is redirected.

pub async fn handle_redirect(
    client: &ConfiguredOAuthClient,
    params: &str,
) -> Result<Agent<ConfiguredOAuthSession>, anyhow::Error> {
    let params = serde_html_form::from_str(params)?;
    let (session, _) = client.callback(params).await?;

    // Get DID from session before creating the agent
    // let did = session
    //     .did()
    //     .ok_or_else(|| anyhow::anyhow!("No DID in OAuth session"))?;
    // let did_string = did.to_string();

    let agent = Agent::new(session);

    Ok(agent)
}
