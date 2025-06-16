mod notes;

pub(crate) use notes::NoteRepository;

#[derive(Debug, thiserror::Error)]
pub(crate) enum RepositoryError {
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}
