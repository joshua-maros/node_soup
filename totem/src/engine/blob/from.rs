use super::{TypedBlob, Blob};
use crate::engine::BlobLayout;

impl From<u8> for TypedBlob {
    fn from(value: u8) -> Self {
        Self {
            blob: Blob {
                bytes: vec![value],
                dynamic_components: vec![],
            },
            layout: BlobLayout::Byte,
        }
    }
}

impl From<i32> for TypedBlob {
    fn from(value: i32) -> Self {
        Self {
            blob: Blob {
                bytes: value.to_ne_bytes().into(),
                dynamic_components: vec![],
            },
            layout: BlobLayout::Integer,
        }
    }
}

impl From<f32> for TypedBlob {
    fn from(value: f32) -> Self {
        Self {
            blob: Blob {
                bytes: value.to_ne_bytes().into(),
                dynamic_components: vec![],
            },
            layout: BlobLayout::Float,
        }
    }
}

impl From<String> for TypedBlob {
    fn from(value: String) -> Self {
        let len = value.len();
        Self {
            blob: Blob {
                bytes: value.into_bytes(),
                dynamic_components: vec![],
            },
            layout: BlobLayout::DynamicIndex(Box::new(BlobLayout::Byte)),
        }
    }
}
