use askama::Template;
use datastar::{consts::FragmentMergeMode, prelude::MergeFragments};
use uuid::Uuid;

use crate::model;

pub(crate) const NOTE_LIST_ID: &str = "#note-list";

#[inline(always)]
pub(crate) fn note_selector(id: &Uuid) -> String {
    format!("#note-{}", id)
}

#[derive(Template)]
#[template(path = "fragments/note.fragment.html")]
pub(crate) struct NoteFragment {
    pub note: model::Note,
}

impl NoteFragment {
    pub(crate) fn fragment(&self) -> Result<MergeFragments, askama::Error> {
        self.render().map(|html| {
            MergeFragments::new(html)
                .selector(self.selector())
                .merge_mode(FragmentMergeMode::Outer)
        })
    }

    pub(crate) fn selector(&self) -> String {
        note_selector(&self.note.id)
    }
}

#[derive(Template)]
#[template(path = "fragments/edit-note.fragment.html")]
pub(crate) struct EditNoteFragment {
    pub note: model::Note,
}

impl EditNoteFragment {
    pub(crate) fn fragment(&self) -> Result<MergeFragments, askama::Error> {
        self.render().map(|html| {
            MergeFragments::new(html)
                .selector(self.selector())
                .merge_mode(FragmentMergeMode::Outer)
        })
    }

    pub(crate) fn selector(&self) -> String {
        note_selector(&self.note.id)
    }
}
