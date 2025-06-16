use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use axum_login::{AuthnBackend, AuthzBackend};
use chrono::{DateTime, TimeDelta, Utc};
use moka::future::Cache;
use openid::{
    Bearer, DiscoveredClient, StandardClaimsSubject, Token, TokenIntrospection,
    error::StandardClaimsSubjectMissing,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::model::{PendingAction, SessionUser, UserId};

#[derive(Debug, Clone)]
pub(crate) struct OidcConfig {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct OidcState {
    pub action: Option<PendingAction>,
}

#[derive(Clone)]
pub(crate) struct OidcAuthBackend {
    client: Arc<DiscoveredClient>,
    scopes: String,

    // In memory store of all the authenticated users
    users: Cache<UserId, SessionUser>,
    //users: Arc<RwLock<HashMap<UserId, SessionUser>>>,

    // In memory story for in flight requests with additional state attached to it
    login_requests: Arc<RwLock<HashMap<Uuid, OidcState>>>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum OidcError {
    #[error(transparent)]
    ClientCreationError(#[from] openid::error::Error),
}

impl OidcAuthBackend {
    pub async fn new(config: OidcConfig) -> Result<Self, OidcError> {
        let issuer = reqwest::Url::parse(&config.issuer_url).unwrap();

        Ok(Self {
            login_requests: Arc::new(RwLock::new(HashMap::new())),
            scopes: config.scopes.join(" "),
            client: Arc::new(
                DiscoveredClient::discover(
                    config.client_id.to_owned(),
                    config.client_secret.to_owned(),
                    Some(config.redirect_url.to_owned()),
                    issuer,
                )
                .await?,
            ),
            users: Cache::builder()
                .initial_capacity(100)
                .max_capacity(64_000)
                .time_to_idle(Duration::from_secs(15 * 60))
                .name("user sessions")
                .build(),
        })
    }

    pub async fn get_authentication_url(&self, state: OidcState) -> String {
        let uuid = Uuid::new_v4();
        self.login_requests.write().await.insert(uuid, state);

        let scopes = Some(self.scopes.as_str());
        let state = uuid.to_string();

        let url = self.client.auth_uri(scopes, state.as_str());
        url.to_string().to_owned()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AuthError {
    #[error("unable to authenticate user")]
    OidcPortalError(String),
}

#[derive(Deserialize)]
pub(crate) struct LoginCallback {
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub state: Option<String>,
    pub code: Option<String>,
    pub iss: Option<String>,
}

#[derive(Deserialize)]
pub(crate) enum AuthenticationCredentials {
    LoginCallback(LoginCallback),
}

// TODO: Create a typed struct
#[derive(Debug, Deserialize, Serialize)]
struct CustomUserInfo(HashMap<String, serde_json::Value>);

impl StandardClaimsSubject for CustomUserInfo {
    fn sub(&self) -> Result<&str, StandardClaimsSubjectMissing> {
        self.0
            .get("sub")
            .and_then(|x| x.as_str())
            .ok_or(StandardClaimsSubjectMissing)
    }
}

impl openid::CompactJson for CustomUserInfo {}

#[async_trait]
impl AuthnBackend for OidcAuthBackend {
    type User = SessionUser;
    type Credentials = AuthenticationCredentials;
    type Error = AuthError;

    #[instrument(skip(self, creds))]
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            AuthenticationCredentials::LoginCallback(callback) => {
                if callback.iss.is_none() {
                    warn!("OIDC callback missing 'iss' field");

                    return Err(AuthError::OidcPortalError(
                        "OIDC callback missing 'iss' field".to_string(),
                    ));
                }

                if let Some(error) = callback.error {
                    warn!(
                        "OIDC login error: {}: {}",
                        error,
                        callback
                            .error_description
                            .unwrap_or("<no description>".to_string())
                    );
                    return Err(AuthError::OidcPortalError(error));
                }

                if callback.state.is_none() {
                    warn!("OIDC callback missing 'state' field");
                    return Err(AuthError::OidcPortalError(
                        "OIDC callback missing 'state' field".to_string(),
                    ));
                }

                let state = callback.state.unwrap();

                if let Some(code) = callback.code {
                    let state = Uuid::parse_str(&state).unwrap_or_else(|_| {
                        warn!("Invalid state in OIDC callback: {}", state);
                        Uuid::new_v4()
                    });

                    let state = match self.login_requests.write().await.remove(&state) {
                        Some(state) => state,
                        None => {
                            warn!("State not found in login requests: {}", state);
                            return Err(AuthError::OidcPortalError("Invalid state".to_string()));
                        }
                    };

                    let bearer = self.client.request_token(&code).await.unwrap();
                    let mut token: Token = bearer.into();

                    if let Some(id_token) = token.id_token.as_mut() {
                        self.client.decode_token(id_token).unwrap();
                        self.client.validate_token(id_token, None, None).unwrap();
                    } else {
                        warn!("Failed validation, no id_token found");
                        return Err(AuthError::OidcPortalError(
                            "failed validation, no id_token found".to_string(),
                        ));
                    }

                    let userinfo: CustomUserInfo =
                        self.client.request_userinfo_custom(&token).await.unwrap();

                    // Verify token vis introspection since no JWT:
                    let introspection: TokenIntrospection<CustomUserInfo> = self
                        .client
                        .request_token_introspection(&token)
                        .await
                        .unwrap();

                    let expiration =
                        DateTime::from_timestamp(introspection.exp.unwrap(), 0).unwrap();
                    let now = Utc::now();

                    if introspection.active == false && expiration > now {
                        return Err(AuthError::OidcPortalError(
                            "User is not active or token has expired".to_string(),
                        ));
                    }

                    // TODO: User ID should something be like blake3(iss + sub) to ensure users of multiple OIDC providers don't collide
                    let user_id = UserId(Uuid::parse_str(userinfo.sub().unwrap()).unwrap_or_else(
                        |_| {
                            warn!("Invalid sub in userinfo: {}", userinfo.sub().unwrap());
                            Uuid::new_v4()
                        },
                    ));

                    let session_user = SessionUser {
                        id: user_id,
                        access_token_hash: blake3::hash(token.bearer.access_token.as_bytes())
                            .as_bytes()
                            .to_vec(),
                        access_token: token.bearer.access_token,
                        pending_action: state.action,
                        expiration: (expiration - now).to_std().unwrap(),
                        last_health_check: now,
                    };

                    self.users.insert(user_id, session_user.clone()).await;

                    return Ok(Some(session_user));
                }

                tracing::warn!("Invalid OIDC flow!");
                return Err(AuthError::OidcPortalError("Invalid OIDC flow!".to_string()));
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_user(&self, user_id: &UserId) -> Result<Option<Self::User>, Self::Error> {
        let user = self.users.get(user_id).await;
        if let Some(user) = user {
            let now = Utc::now();

            if (user.last_health_check + TimeDelta::seconds(10)) < now {
                info!("Performing token introspection");

                let token: Token = Bearer {
                    access_token: user.access_token.clone(),
                    token_type: "bearer".to_string(),
                    scope: None,
                    state: None,
                    refresh_token: None,
                    expires_in: None,
                    id_token: None,
                    extra: None,
                }
                .into();

                let introspection: TokenIntrospection<CustomUserInfo> = self
                    .client
                    .request_token_introspection(&token)
                    .await
                    .unwrap();

                info!("introspection: {:?}", introspection);

                let expiration = DateTime::from_timestamp(introspection.exp.unwrap(), 0).unwrap();

                if introspection.active == false && expiration > now {
                    warn!("Oidc introspection resulted in invalid token for user!");
                    let _ = self.users.remove(user_id).await;

                    Err(AuthError::OidcPortalError(
                        "User is not active or token has expired".to_string(),
                    ))
                } else {
                    // Update cache
                    let new_user = SessionUser {
                        id: user.id,
                        access_token: user.access_token,
                        access_token_hash: user.access_token_hash,
                        pending_action: user.pending_action,
                        expiration: user.expiration,
                        last_health_check: now,
                    };

                    self.users.insert(*user_id, new_user.clone()).await;

                    Ok(Some(new_user))
                }
            } else {
                Ok(Some(user))
            }
        } else {
            Ok(user)
        }
    }
}

#[async_trait]
impl AuthzBackend for OidcAuthBackend {
    type Permission = String;
}
