use super::Blob;
use crate::engine::ObjectLayout;

impl Blob {
    pub fn layout(&self) -> &ObjectLayout {
        self.view().layout()
    }
}
