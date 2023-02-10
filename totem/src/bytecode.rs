pub struct Heap {
    data: Vec<u32>,
    usage: Vec<Usage>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Usage {
    Bool,
    Integer,
    Float,
    Allocated,
    Free,
}

macro_rules! put_fn {
    ($name:ident, $Type:ty, $usage:expr) => {
        pub fn $name(&mut self, index: usize, data: $Type) {
            unsafe { self.set_data(index, u32::from_ne_bytes(data.to_ne_bytes()), $usage) }
        }
    };
}

macro_rules! get_fn {
    ($name:ident, $Type:ty, $usage:expr) => {
        pub fn $name(&mut self, index: usize) -> $Type {
            debug_assert_eq!(self.usage[index], $usage);
            <$Type>::from_ne_bytes(self.data[index].to_ne_bytes())
        }
    };
}

macro_rules! clear_fn {
    ($name:ident, $usage:expr) => {
        pub fn $name(&mut self, index: usize) {
            debug_assert_eq!(self.usage[index], $usage);
            self.usage[index] = Usage::Free;
        }
    };
}

impl Heap {
    put_fn!(put_integer, i32, Usage::Integer);

    put_fn!(put_float, f32, Usage::Float);

    get_fn!(get_integer, i32, Usage::Integer);

    get_fn!(get_float, f32, Usage::Float);

    clear_fn!(clear_integer, Usage::Integer);

    clear_fn!(clear_float, Usage::Float);

    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            usage: vec![Usage::Free; size],
        }
    }

    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0);
        self.usage.resize(new_size, Usage::Free);
    }

    /// This does NOT zero the data, because to allocate data you have to
    /// specify its initial value, automatically overwriting whatever garbage
    /// was left by the last clear.
    pub fn clear(&mut self) {
        self.usage.fill(Usage::Free);
    }

    /// May grow the buffer to make room.
    pub fn allocate_space_for_single_value(&mut self) -> usize {
        let old_size = self.usage.len();
        let free_index = (0..old_size).find(|index| self.usage[*index] == Usage::Free);
        let free_index = free_index.unwrap_or_else(|| {
            self.resize(old_size + 1);
            old_size
        });
        self.usage[free_index] = Usage::Allocated;
        free_index
    }

    /// May grow the buffer to make room.
    pub fn allocate_space_for_multiple_values(&mut self, count: usize) -> usize {
        if count == 0 {
            return 0;
        }
        let old_size = self.usage.len();
        'check_next_position: for index in count - 1..old_size {
            for offset in 0..count {
                if self.usage[index - offset] != Usage::Free {
                    continue 'check_next_position;
                }
            }
            // We only get here if the range is completely free.
            for offset in 0..count {
                self.usage[index - offset] = Usage::Allocated;
            }
            return index - (count - 1);
        }
        self.resize(old_size + count);
        for offset in 0..count {
            self.usage[old_size + offset] = Usage::Allocated;
        }
        old_size
    }

    pub unsafe fn set_data(&mut self, index: usize, data: u32, usage: Usage) {
        self.data[index] = data;
        self.usage[index] = usage;
    }
}

pub enum BytecodeInstruction {
    UnaryOp {
        op: UnaryOp,
        input: usize,
        output: usize,
    },
    BinaryOp {
        op: BinaryOp,
        input_1: usize,
        input_2: usize,
        output: usize,
    },
    IntegerLiteral(i32, usize),
    FloatLiteral(f32, usize),
    Copy(usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UnaryOp {
    CastIntToFloat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryOp {
    FloatOp(FloatOp),
    IntegerOp(IntegerOp),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FloatOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IntegerOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BytecodeInstruction {
    fn execute(&self, heap: &mut Heap) {
        match self {
            &BytecodeInstruction::UnaryOp { op, input, output } => match op {
                UnaryOp::CastIntToFloat => {
                    let result = heap.get_integer(input) as f32;
                    heap.put_float(output, result);
                }
            },
            &BytecodeInstruction::BinaryOp {
                op,
                input_1,
                input_2,
                output,
            } => match op {
                BinaryOp::FloatOp(op) => {
                    let input_1 = heap.get_float(input_1);
                    let input_2 = heap.get_float(input_2);
                    let result = match op {
                        FloatOp::Add => input_1 + input_2,
                        FloatOp::Subtract => input_1 - input_2,
                        FloatOp::Multiply => input_1 * input_2,
                        FloatOp::Divide => input_1 / input_2,
                    };
                    heap.put_float(output, result);
                }
                BinaryOp::IntegerOp(op) => {
                    let input_1 = heap.get_integer(input_1);
                    let input_2 = heap.get_integer(input_2);
                    let result = match op {
                        IntegerOp::Add => input_1 + input_2,
                        IntegerOp::Subtract => input_1 - input_2,
                        IntegerOp::Multiply => input_1 * input_2,
                        IntegerOp::Divide => input_1 / input_2,
                    };
                    heap.put_integer(output, result);
                }
            },
        }
    }
}

pub struct BytecodeProgram {
    instructions: Vec<BytecodeInstruction>,
}

impl BytecodeProgram {
    pub fn new(instructions: Vec<BytecodeInstruction>) -> Self {
        Self { instructions }
    }

    pub fn min_heap_size(&self) -> usize {
        let mut min_size = 0;
        for instruction in &self.instructions {
            match instruction {
                &BytecodeInstruction::UnaryOp { input, output, .. } => {
                    min_size = min_size.max(input).max(output)
                }
                &BytecodeInstruction::BinaryOp {
                    input_1,
                    input_2,
                    output,
                    ..
                } => min_size = min_size.max(input_1).max(input_2).max(output),
            }
        }
        min_size
    }

    pub fn execute(&self, heap: &mut Heap) {
        for i in &self.instructions {
            i.execute(heap);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MemoryLayout {
    Integer(usize),
    Float(usize),
    Struct { components: Vec<(String, MemoryLayout)> },
}
