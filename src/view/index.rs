use askama::Template;
use axum::{extract::State, response::Html};

use crate::{auth::AuthSession, model, service::{NoteService}};

#[derive(Template)]
#[template(path = "index.html")]
pub(crate) struct Index {
    title: String,
    partial: bool,
    notes: Vec<model::Note>,
}

pub(crate) async fn index(State(notes): State<NoteService>, auth_session: AuthSession) -> Html<String> {
    let user = auth_session.user.unwrap();

    Html(
        Index {
            title: "TodoList".to_owned(),
            partial: false,
            notes: notes.get_notes(user.id).await.unwrap(),
        }
        .render()
        .unwrap(),
    )
}
