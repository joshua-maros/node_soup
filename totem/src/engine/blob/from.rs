use super::{Blob, RawObject};
use crate::engine::ObjectLayout;

impl From<u8> for Blob {
    fn from(value: u8) -> Self {
        Self {
            object: RawObject {
                bytes: vec![value],
                dynamic_components: vec![],
            },
            layout: ObjectLayout::Byte,
        }
    }
}

impl From<i32> for Blob {
    fn from(value: i32) -> Self {
        Self {
            object: RawObject {
                bytes: value.to_ne_bytes().into(),
                dynamic_components: vec![],
            },
            layout: ObjectLayout::Integer,
        }
    }
}

impl From<f32> for Blob {
    fn from(value: f32) -> Self {
        Self {
            object: RawObject {
                bytes: value.to_ne_bytes().into(),
                dynamic_components: vec![],
            },
            layout: ObjectLayout::Float,
        }
    }
}

impl From<String> for Blob {
    fn from(value: String) -> Self {
        let len = value.len();
        Self {
            object: RawObject {
                bytes: value.into_bytes(),
                dynamic_components: vec![],
            },
            layout: ObjectLayout::DynamicIndex(Box::new(ObjectLayout::Byte)),
        }
    }
}