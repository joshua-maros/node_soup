use std::{fmt::Debug, hash::Hash, marker::PhantomData};

pub fn pretty_format_number(number: f32) -> String {
    if !number.is_finite() {
        format!("{}", number)
    } else if number == 0.0 {
        format!("0.000")
    } else {
        let mut number = number;
        let mut power = 0;
        while number.abs() >= 1000.0 {
            power += 1;
            number /= 1000.0;
        }
        while number.abs() < 0.01 {
            power -= 1;
            number *= 1000.0;
        }
        let suffix = match power {
            4 => "T",
            3 => "G",
            2 => "M",
            1 => "k",
            0 => "",
            -1 => "m",
            -2 => "μ",
            -3 => "n",
            -4 => "p",
            _ => "?",
        };
        let len = if number < 0.0 { 6 } else { 5 };
        format!("{}{}", &format!("{:0.4}", number)[..len], suffix)
    }
}

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id<Of>(u32, PhantomData<Of>);

impl<Of> Clone for Id<Of> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<Of> Copy for Id<Of> {}

impl<Of> Debug for Id<Of> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("{}Id", std::any::type_name::<Of>()))
            .field(&self.0)
            .finish()
    }
}

impl<Of> PartialEq for Id<Of> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Of> Eq for Id<Of> {}

impl<Of> Hash for Id<Of> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub struct IdCreator<Of> {
    next: Id<Of>,
}

impl<Of> IdCreator<Of> {
    pub fn new() -> Self {
        Self {
            next: Id(0, PhantomData),
        }
    }

    pub fn next(&mut self) -> Id<Of> {
        let id = self.next;
        self.next.0 += 1;
        id
    }
}
