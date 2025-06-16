use std::{
    fmt::{Debug, Display},
    time::Duration,
};

use axum_login::AuthUser;
use chrono::{DateTime, Utc};
use moka::Expiry;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub(crate) type NoteId = Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Note {
    pub id: NoteId,
    pub owner: UserId,
    pub content: String,
    pub checked: bool,
}

#[derive(
    Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    CheckNote(NoteId),
    UncheckNote(NoteId),
    EditNote(NoteId, String),
    DeleteNote(NoteId),
    CreateNote(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApplicationSignals {
    pub note: String,
}

#[derive(Clone)]
pub struct SessionUser {
    pub id: UserId,
    pub access_token: String,
    pub access_token_hash: Vec<u8>,
    pub pending_action: Option<PendingAction>,
    pub expiration: Duration,
    pub last_health_check: DateTime<Utc>,
}

impl Debug for SessionUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionUser")
            .field("id", &self.id)
            .field("access_token", &"<redacted>")
            .field("access_token_hash", &"<redacted>")
            .field("pending_action", &self.pending_action)
            .field("expiration", &self.expiration)
            .field("last_health_check", &self.last_health_check)
            .finish()
    }
}

impl Expiry<UserId, SessionUser> for SessionUser {
    fn expire_after_create(
        &self,
        _: &UserId,
        value: &SessionUser,
        _: std::time::Instant,
    ) -> Option<Duration> {
        Some(value.expiration)
    }
}

impl AuthUser for SessionUser {
    type Id = UserId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.access_token_hash
    }
}
