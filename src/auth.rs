use std::str::FromStr;

use async_stream::stream;
use axum::{
    extract::{Query, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Redirect},
};
use datastar::{Sse, axum::ReadSignals, consts::FragmentMergeMode, prelude::MergeFragments};
use regex::Regex;
use serde::Deserialize;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    model::{ApplicationSignals, PendingAction},
    service::{AuthenticationCredentials, LoginCallback, NoteService, OidcAuthBackend, OidcState},
};

pub(crate) type AuthSession = axum_login::AuthSession<OidcAuthBackend>;

pub(crate) async fn login_callback(
    mut auth_session: AuthSession,
    Query(query): Query<LoginCallback>,
    State(notes): State<NoteService>,
) -> impl IntoResponse {
    if auth_session.user.is_some() {
        info!("User is already authenticated, redirecting to home page.");
        return Redirect::temporary("/login").into_response();
    }

    return match auth_session
        .authenticate(AuthenticationCredentials::LoginCallback(query))
        .await
    {
        Ok(user) => {
            let mut user = user.unwrap();

            if let Err(err) = auth_session.login(&user).await {
                error!("Failed to login user: {:?}", err);

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            } else {
                // TODO: Find a better spot for this:
                if let Some(action) = user.pending_action {
                    debug!("Processing pending action for user");

                    match action {
                        PendingAction::CheckNote(note_id) => {
                            let _ = notes.update_note_checked(user.id, note_id, true).await;
                        }
                        PendingAction::UncheckNote(note_id) => {
                            let _ = notes.update_note_checked(user.id, note_id, false).await;
                        }
                        PendingAction::EditNote(note_id, content) => {
                            let _ = notes.update_note_content(user.id, note_id, &content).await;
                        }
                        PendingAction::DeleteNote(note_id) => {
                            let _ = notes.delete_note(user.id, note_id).await;
                        }
                        PendingAction::CreateNote(content) => {
                            let _ = notes.create_note(user.id, &content).await;
                        }
                    };

                    user.pending_action = None;
                }

                Redirect::temporary("/").into_response()
            }
        }
        Err(e) => {
            warn!("Authentication failed: {:?}", e);
            Redirect::temporary("/login/error").into_response()
        }
    };
}

pub(crate) async fn login(auth_session: AuthSession) -> impl IntoResponse {
    if auth_session.user.is_some() {
        Redirect::temporary("/").into_response()
    } else {
        Redirect::temporary(
            auth_session
                .backend
                .get_authentication_url(OidcState::default())
                .await
                .as_str(),
        )
        .into_response()
    }
}

#[derive(Deserialize)]
pub(crate) struct NextQuery {
    next: Option<String>,
}

pub(crate) async fn login_datastar(
    method: Method,
    auth_session: AuthSession,
    Query(query): Query<NextQuery>,
    ReadSignals(signals): ReadSignals<ApplicationSignals>,
) -> impl IntoResponse {
    let uri = if let Some(next) = query.next {
        // Convert query parameter to respective action
        let regex = Regex::new(
            r"^\/note\/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})(\/:(check|uncheck))?$",
        ).unwrap();

        let action = if let Some(capture_group) = regex.captures(&next) {
            let id = Uuid::from_str(capture_group.get(1).unwrap().as_str()).unwrap();

            match capture_group.get(3).map(|m| m.as_str()).unwrap_or("") {
                "check" => Some(PendingAction::CheckNote(id)),
                "uncheck" => Some(PendingAction::UncheckNote(id)),
                "" if method == Method::DELETE => Some(PendingAction::DeleteNote(id)),
                "" if method == Method::PUT => Some(PendingAction::EditNote(id, signals.note)),

                _ => {
                    warn!("Couldn't parse the next query parameter: {}", next);
                    None
                },
            }
        } else if next == "/note" {
            Some(PendingAction::CreateNote(signals.note))
        } else {
            // Should this even be possible?
            warn!("Login redirect for a PUT/POST/DELETE method, but no action!");
            None
        };

        auth_session
            .backend
            .get_authentication_url(OidcState { action })
            .await
    } else {
        "/login".to_string()
    };

    // Redirection with a meta tag to avoid CSP issues
    // TODO: Can this be handled via a pop-up window?
    Sse(stream! {
        yield MergeFragments::new(format!("<meta http-equiv='Refresh' content='0; URL={uri}'/>"))
            .merge_mode(FragmentMergeMode::Append)
            .selector("head")
            .into();
    })
    .into_response()
}

pub(crate) async fn login_error() -> impl IntoResponse {
    "Login error, please try again.".into_response()
}
