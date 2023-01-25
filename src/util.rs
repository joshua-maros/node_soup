pub fn pretty_format_number(number: f32) -> String {
    if !number.is_finite() {
        format!("{}", number)
    } else if number == 0.0 {
        format!("0.00")
    } else {
        let mut number = number;
        let mut power = 0;
        while number >= 1000.0 {
            power += 1;
            number /= 1000.0;
        }
        while number < 0.01 {
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
        format!("{}{}", &format!("{:0.3}", number)[..4], suffix)
    }
}
