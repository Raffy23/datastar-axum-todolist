use tracing::error;

use crate::{
    model::{Note, NoteId, UserId},
    repository::NoteRepository,
};

#[derive(Debug, Clone)]
pub(crate) struct NoteService {
    repository: NoteRepository,
}

// TODO: Improve error handling
impl NoteService {
    pub(crate) fn new(repository: NoteRepository) -> Self {
        Self { repository }
    }

    pub async fn create_note(&self, user_id: UserId, content: &str) -> Result<Note, ()> {
        self.repository
            .create(user_id, content)
            .await
            .map_err(|error| error!("Failed to create note: {:?}", error))
            .map(|id| Note {
                id,
                owner: user_id,
                content: content.to_string(),
                checked: false,
            })
    }

    pub async fn get_note(&self, user_id: UserId, id: NoteId) -> Result<Note, ()> {
        self.repository
            .find_by_id(user_id, id)
            .await
            .map_err(|error| error!("Failed to get note: {:?}", error))
    }

    pub async fn get_notes(&self, user_id: UserId) -> Result<Vec<Note>, ()> {
        self.repository
            .find_all(user_id)
            .await
            .map_err(|error| error!("Failed to get notes: {:?}", error))
    }

    pub async fn update_note_content(
        &self,
        user_id: UserId,
        id: NoteId,
        content: &str,
    ) -> Result<Note, ()> {
        self.repository
            .update_content(user_id, id, content)
            .await
            .map_err(|error| error!("Failed to update note: {:?}", error))
            .unwrap();

        return self.get_note(user_id, id).await;
    }

    pub async fn update_note_checked(
        &self,
        user_id: UserId,
        id: NoteId,
        checked: bool,
    ) -> Result<Note, ()> {
        self.repository
            .update_checked(user_id, id, checked)
            .await
            .map_err(|error| error!("Failed to update note: {:?}", error))
            .unwrap();

        return self.get_note(user_id, id).await;
    }

    pub async fn delete_note(&self, user_id: UserId, id: NoteId) -> Result<u64, ()> {
        self.repository
            .delete(user_id, id)
            .await
            .map_err(|error| error!("Failed to delete note: {:?}", error))
    }
}
