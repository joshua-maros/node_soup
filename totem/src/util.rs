use std::{cmp::Ordering, fmt::Debug, hash::Hash, marker::PhantomData};

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
        while number.abs() < 1.0 {
            power -= 1;
            number *= 1000.0;
        }
        let suffix = match power {
            11 => "Dc",
            10 => "Nn",
            9 => "Oc",
            8 => "Sp",
            7 => "Sx",
            6 => "Qn",
            5 => "Qd",
            4 => "Tl",
            3 => "Bl",
            2 => "Ml",
            1 => "Th",
            0 => "",
            -1 => "tht",
            -2 => "mlt",
            -3 => "blt",
            -4 => "tlt",
            -5 => "qdc",
            -6 => "qnt",
            -7 => "sxt",
            -8 => "spt",
            -9 => "oct",
            -10 => "nnt",
            -11 => "dct",
            _ => "?",
        };
        let len = if number < 0.0 { 6 } else { 5 };
        if suffix == "?" {
            format!(
                "{}×10{}",
                &format!("{:0.4}", number)[..len],
                superscript_format_number(3 * power)
            )
        } else {
            format!("{}{}", &format!("{:0.4}", number)[..len], suffix)
        }
    }
}

pub fn superscript_format_number(num: i32) -> String {
    let mut result = format!("{}", num);
    for (original, replacement) in "-0123456789".chars().zip("⁻⁰¹²³⁴⁵⁶⁷⁸⁹".chars())
    {
        result = result.replace(original, &format!("{}", replacement));
    }
    result
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

impl<Of> PartialOrd for Id<Of> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<Of> Ord for Id<Of> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
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
