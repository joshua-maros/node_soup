use itertools::Itertools;

use super::{TypedBlob, TypedBlobView, Blob};
use crate::engine::BlobLayout;

impl TypedBlob {
    pub fn fixed_heterogeneous_map(components: Vec<(TypedBlob, TypedBlob)>) -> Self {
        let len = components.len() as u32;
        assert!(len > 0);
        let mut keys = Vec::new();
        let mut value_layouts = Vec::new();
        let mut bytes = Vec::new();
        let mut children = Vec::new();
        for (key, mut value) in components.into_iter() {
            keys.push(key);
            if value.layout.is_dynamic() {
                bytes.append(&mut vec![0; std::mem::size_of::<usize>()]);
                children.push(value.blob);
            } else {
                bytes.append(&mut value.blob.bytes);
                children.append(&mut value.blob.dynamic_components);
            }
            value_layouts.push(value.layout);
        }
        Self {
            blob: Blob {
                bytes,
                dynamic_components: children,
            },
            layout: BlobLayout::FixedHeterogeneousMap(
                Box::new(TypedBlob::fixed_array(keys)),
                value_layouts,
            ),
        }
    }

    pub fn dynamic_array(values: Vec<TypedBlob>) -> Self {
        let len = values.len() as u32;
        assert!(len > 0);
        let layout = &values[0].layout;
        for value in &values[1..] {
            assert_eq!(&value.layout, layout);
        }
        if layout.is_dynamic() {
            Self {
                layout: BlobLayout::DynamicIndex(Box::new(layout.clone())),
                blob: Blob {
                    bytes: vec![0; values.len() * std::mem::size_of::<usize>()],
                    dynamic_components: values.into_iter().map(|value| value.blob).collect_vec(),
                },
            }
        } else {
            let mut bytes = Vec::new();
            let mut children = Vec::new();
            let layout = BlobLayout::DynamicIndex(Box::new(layout.clone()));
            let mut values = values;
            for value in &mut values {
                bytes.append(&mut value.blob.bytes);
                children.append(&mut value.blob.dynamic_components);
            }
            Self {
                blob: Blob {
                    bytes,
                    dynamic_components: children,
                },
                layout,
            }
        }
    }

    pub fn fixed_array(values: Vec<TypedBlob>) -> Self {
        let len = values.len() as u32;
        assert!(len > 0);
        let layout = &values[0].layout;
        for value in &values[1..] {
            assert_eq!(&value.layout, layout);
        }
        if layout.is_dynamic() {
            Self {
                layout: BlobLayout::FixedIndex(len, Box::new(layout.clone())),
                blob: Blob {
                    bytes: vec![0; values.len() * std::mem::size_of::<usize>()],
                    dynamic_components: values.into_iter().map(|value| value.blob).collect_vec(),
                },
            }
        } else {
            let mut bytes = Vec::new();
            let mut dynamic_components = Vec::new();
            let layout = BlobLayout::FixedIndex(len, Box::new(layout.clone()));
            let mut values = values;
            for value in &mut values {
                bytes.append(&mut value.blob.bytes);
                dynamic_components.append(&mut value.blob.dynamic_components);
            }
            Self {
                blob: Blob {
                    bytes,
                    dynamic_components,
                },
                layout,
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SafetyLock(());

impl<'a> TypedBlobView<'a> {
    pub(super) unsafe fn new(
        layout: &'a BlobLayout,
        bytes: &'a [u8],
        dynamic_components: &'a [Blob],
    ) -> Self {
        Self {
            layout,
            bytes,
            dynamic_components,
            safety_lock: SafetyLock(()),
        }
    }
}
