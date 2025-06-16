use async_stream::stream;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use datastar::{
    Sse,
    axum::ReadSignals,
    consts::FragmentMergeMode,
    prelude::{MergeSignals, RemoveFragments},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    fragments::{EditNoteFragment, NOTE_LIST_ID, NoteFragment, note_selector},
    service::NoteService,
};

pub(crate) async fn delete_note(
    Path(id): Path<Uuid>,
    State(notes): State<NoteService>,
    auth_session: AuthSession,
) -> impl IntoResponse {
    let user = auth_session
        .user
        .expect("User must be logged in to use this endpoint");

    notes.delete_note(user.id, id).await.unwrap();

    Sse(stream! {
        yield RemoveFragments::new(note_selector(&id)).into();
    })
}

pub(crate) async fn edit_note_view(
    Path(id): Path<Uuid>,
    State(notes): State<NoteService>,
    auth_session: AuthSession,
) -> impl IntoResponse {
    let user = auth_session
        .user
        .expect("User must be logged in to use this endpoint");

    let note = EditNoteFragment {
        note: notes.get_note(user.id, id).await.unwrap(),
    };

    Sse(stream! {
        yield note
            .fragment()
            .unwrap()
            .into();
    })
}

pub(crate) async fn get_note(
    Path(id): Path<Uuid>,
    State(notes): State<NoteService>,
    auth_session: AuthSession,
) -> impl IntoResponse {
    let user = auth_session
        .user
        .expect("User must be logged in to use this endpoint");

    let note = NoteFragment {
        note: notes.get_note(user.id, id).await.unwrap(),
    };

    Sse(stream! {
        yield note
            .fragment()
            .unwrap()
            .into();
    })
}

#[derive(Deserialize)]
pub(crate) struct UpdateSignals {
    pub content: String,
}

pub(crate) async fn update_note(
    Path(id): Path<Uuid>,
    State(notes): State<NoteService>,
    auth_session: AuthSession,
    ReadSignals(signals): ReadSignals<UpdateSignals>,
) -> impl IntoResponse {
    let user = auth_session
        .user
        .expect("User must be logged in to use this endpoint");

    let note = notes
        .update_note_content(user.id, id, &signals.content)
        .await
        .unwrap();

    let note = NoteFragment { note: note };

    Sse(stream! {
        yield note
            .fragment()
            .unwrap()
            .into();
    })
}

#[derive(Deserialize)]
pub(crate) struct NewNoteSignals {
    pub note: String,
}

pub(crate) async fn new_note(
    State(notes): State<NoteService>,
    auth_session: AuthSession,
    ReadSignals(signals): ReadSignals<NewNoteSignals>,
) -> impl IntoResponse {
    let user = auth_session
        .user
        .expect("User must be logged in to use this endpoint");

    let note = notes.create_note(user.id, &signals.note).await.unwrap();

    let note = NoteFragment { note: note };

    Sse(stream! {
        // Clear the input field for the note
        yield MergeSignals::new("{ note: '' }").into();

        // Send the new note to the client
        yield note
            .fragment()
            .unwrap()
            .selector(NOTE_LIST_ID)
            .merge_mode(FragmentMergeMode::Append)
            .into();
    })
}

pub(crate) async fn check_note(
    Path(id): Path<Uuid>,
    State(notes): State<NoteService>,
    auth_session: AuthSession,
) -> impl IntoResponse {
    let user = auth_session
        .user
        .expect("User must be logged in to use this endpoint");

    let note = notes
        .update_note_checked(user.id, id, true)
        .await
        .unwrap();

    let note = NoteFragment { note: note };

    Sse(stream! {
        yield note
            .fragment()
            .unwrap()
            .into();
    })
}

pub(crate) async fn uncheck_note(
    Path(id): Path<Uuid>,
    State(notes): State<NoteService>,
    auth_session: AuthSession,
) -> impl IntoResponse {
    let user = auth_session
        .user
        .expect("User must be logged in to use this endpoint");

    let note = notes
        .update_note_checked(user.id, id, false)
        .await
        .unwrap();

    let note = NoteFragment { note: note };

    Sse(stream! {
        yield note
            .fragment()
            .unwrap()
            .into();
    })
}
