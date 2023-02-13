use itertools::Itertools;

use super::{Blob, BlobView, RawObject};
use crate::engine::DataLayout;

impl Blob {
    pub fn fixed_heterogeneous_map(components: Vec<(Blob, Blob)>) -> Self {
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
                children.push(value.object);
            } else {
                bytes.append(&mut value.object.bytes);
                children.append(&mut value.object.dynamic_components);
            }
            value_layouts.push(value.layout);
        }
        Self {
            object: RawObject {
                bytes,
                dynamic_components: children,
            },
            layout: DataLayout::FixedHeterogeneousMap(
                Box::new(Blob::fixed_array(keys)),
                value_layouts,
            ),
        }
    }

    pub fn dynamic_array(values: Vec<Blob>) -> Self {
        let len = values.len() as u32;
        assert!(len > 0);
        let layout = &values[0].layout;
        for value in &values[1..] {
            assert_eq!(&value.layout, layout);
        }
        if layout.is_dynamic() {
            Self {
                layout: DataLayout::DynamicIndex(Box::new(layout.clone())),
                object: RawObject {
                    bytes: vec![0; values.len() * std::mem::size_of::<usize>()],
                    dynamic_components: values.into_iter().map(|value| value.object).collect_vec(),
                },
            }
        } else {
            let mut bytes = Vec::new();
            let mut children = Vec::new();
            let layout = DataLayout::DynamicIndex(Box::new(layout.clone()));
            let mut values = values;
            for value in &mut values {
                bytes.append(&mut value.object.bytes);
                children.append(&mut value.object.dynamic_components);
            }
            Self {
                object: RawObject {
                    bytes,
                    dynamic_components: children,
                },
                layout,
            }
        }
    }

    pub fn fixed_array(values: Vec<Blob>) -> Self {
        let len = values.len() as u32;
        assert!(len > 0);
        let layout = &values[0].layout;
        for value in &values[1..] {
            assert_eq!(&value.layout, layout);
        }
        if layout.is_dynamic() {
            Self {
                layout: DataLayout::FixedIndex(len, Box::new(layout.clone())),
                object: RawObject {
                    bytes: vec![0; values.len() * std::mem::size_of::<usize>()],
                    dynamic_components: values.into_iter().map(|value| value.object).collect_vec(),
                },
            }
        } else {
            let mut bytes = Vec::new();
            let mut dynamic_components = Vec::new();
            let layout = DataLayout::FixedIndex(len, Box::new(layout.clone()));
            let mut values = values;
            for value in &mut values {
                bytes.append(&mut value.object.bytes);
                dynamic_components.append(&mut value.object.dynamic_components);
            }
            Self {
                object: RawObject {
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

impl<'a> BlobView<'a> {
    pub(super) unsafe fn new(
        layout: &'a DataLayout,
        bytes: &'a [u8],
        dynamic_components: &'a [RawObject],
    ) -> Self {
        Self {
            layout,
            bytes,
            dynamic_components,
            safety_lock: SafetyLock(()),
        }
    }
}
