use axum::extract::FromRef;
use sqlx::{Pool, Sqlite};

use crate::{repository::NoteRepository, service::NoteService};

#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    notes: NoteService,
}

impl AppState {
    pub async fn from_database(db: Pool<Sqlite>) -> Self {
        let note_repository = NoteRepository::new(db.clone());
        let notes = NoteService::new(note_repository);

        Self { notes }
    }
}
