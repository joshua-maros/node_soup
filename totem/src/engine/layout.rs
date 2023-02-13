use super::Blob;

#[derive(Clone, Debug, PartialEq)]
pub enum DataLayout {
    Float,
    Integer,
    Byte,
    FixedIndex(u32, Box<DataLayout>),
    DynamicIndex(Box<DataLayout>),
    FixedHeterogeneousMap(Box<Blob>, Vec<DataLayout>),
    FixedHomogeneousMap(Box<Blob>, u32, Box<DataLayout>),
    DynamicMap(Box<DataLayout>),
}

impl DataLayout {
    /// Returns if a value matching this layout can be resized. Does not tell
    /// you anything about whether or not components of this value can be
    /// resized.
    pub fn is_dynamic(&self) -> bool {
        match self {
            DataLayout::Float | DataLayout::Integer | DataLayout::Byte => false,
            DataLayout::FixedIndex(_, base) | DataLayout::FixedHomogeneousMap(_, _, base) => false,
            DataLayout::FixedHeterogeneousMap(_, base) => false,
            DataLayout::DynamicIndex(_) | DataLayout::DynamicMap(_) => true,
        }
    }

    /// Equivalent to `!self.is_dynamic()`
    pub fn is_fixed(&self) -> bool {
        !self.is_dynamic()
    }

    /// Returns the number of elements in the topmost collection this layout
    /// describes. If you want the total size of the structure, use frozen_size
    /// instead.
    pub fn len(&self) -> Option<u32> {
        match self {
            DataLayout::Float | DataLayout::Integer | DataLayout::Byte => None,
            DataLayout::FixedIndex(len, _) => Some(*len),
            DataLayout::DynamicIndex(_) => todo!(),
            DataLayout::FixedHeterogeneousMap(keys, _) => Some(keys.layout().len().unwrap()),
            DataLayout::FixedHomogeneousMap(_, num_keys, _) => Some(*num_keys),
            DataLayout::DynamicMap(_) => todo!(),
        }
    }

    pub fn string_keys(&self) -> Option<Vec<&str>> {
        match self {
            DataLayout::Float
            | DataLayout::Integer
            | DataLayout::Byte
            | DataLayout::FixedIndex(_, _)
            | DataLayout::DynamicIndex(_)
            | DataLayout::DynamicMap(_) => None,
            DataLayout::FixedHeterogeneousMap(keys, _)
            | DataLayout::FixedHomogeneousMap(keys, _, _) => {
                let mut string_keys = Vec::new();
                let keys = keys.view();
                for index in 0..keys.layout().len().unwrap() {
                    string_keys.push(keys.index(&(index as i32).into()).as_string().ok()?)
                }
                Some(string_keys)
            }
        }
    }

    /// How many bytes are needed to store a piece of data in this layout, where
    /// dynamic values are stored as native-width pointers.
    pub fn frozen_size(&self) -> u32 {
        match self {
            DataLayout::Byte => 1,
            DataLayout::Float | DataLayout::Integer => 4,
            DataLayout::FixedIndex(size, eltype)
            | DataLayout::FixedHomogeneousMap(_, size, eltype) => *size * eltype.frozen_size(),
            DataLayout::FixedHeterogeneousMap(_, eltypes) => {
                eltypes.iter().map(|eltype| eltype.frozen_size()).sum()
            }
            DataLayout::DynamicMap(eltype) | DataLayout::DynamicIndex(eltype) => {
                std::mem::size_of::<usize>() as u32
            }
        }
    }

    pub fn layout_after_index(&self, fixed_index: Option<&Blob>) -> Option<&DataLayout> {
        match self {
            DataLayout::Float | DataLayout::Integer | DataLayout::Byte => {
                panic!("Cannot index value of scalar type {:#?}", self)
            }
            DataLayout::FixedIndex(_, eltype)
            | DataLayout::DynamicIndex(eltype)
            | DataLayout::FixedHomogeneousMap(_, _, eltype)
            | DataLayout::DynamicMap(eltype) => Some(&*eltype),
            DataLayout::FixedHeterogeneousMap(keys, eltypes) => {
                let keys = keys.view();
                if let Some(fixed_index) = fixed_index {
                    let fixed_index = fixed_index.view();
                    let mut options = Vec::new();
                    for key_index in 0..keys.len().unwrap() {
                        let key = keys.index(&Blob::from(key_index as i32));
                        if fixed_index == key {
                            return Some(&eltypes[key_index as usize]);
                        } else {
                            options.push(key);
                        }
                    }
                    panic!(
                        "Invalid index {:#?}, options are {:#?}",
                        fixed_index, options
                    );
                } else {
                    None
                }
            }
        }
    }

    pub fn default_blob(&self) -> Blob {
        match self {
            DataLayout::Float => 0.0.into(),
            DataLayout::Integer => 0.into(),
            DataLayout::Byte => 0u8.into(),
            DataLayout::FixedIndex(_, _) => todo!(),
            DataLayout::DynamicIndex(_) => todo!(),
            DataLayout::FixedHeterogeneousMap(keys_blob, eltypes) => {
                let mut components = Vec::new();
                for index in 0..keys_blob.view().len().unwrap() {
                    let name = keys_blob.view().index(&(index as i32).into()).to_owned();
                    components.push((name, eltypes[index as usize].default_blob()));
                }
                Blob::fixed_heterogeneous_map(components)
            }
            DataLayout::FixedHomogeneousMap(_, _, _) => todo!(),
            DataLayout::DynamicMap(_) => todo!(),
        }
    }
}
