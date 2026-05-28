/// Formats a float with the given number of decimal places.
pub fn format_float(val: f64, precision: usize) -> String {
    format!("{val:.precision$}")
}

/// Returns the integer as a string, or an empty string for `None`.
pub fn int_str(val: Option<i64>) -> String {
    match val {
        Some(n) => n.to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_float_two_decimals() {
        assert_eq!(format_float(std::f64::consts::PI, 2), "3.14");
    }

    #[test]
    fn format_float_zero_decimals() {
        assert_eq!(format_float(42.7, 0), "43");
    }

    #[test]
    fn format_float_four_decimals() {
        assert_eq!(format_float(1.5, 4), "1.5000");
    }

    #[test]
    fn int_str_some() {
        assert_eq!(int_str(Some(42)), "42");
    }

    #[test]
    fn int_str_none() {
        assert_eq!(int_str(None), "");
    }

    #[test]
    fn int_str_negative() {
        assert_eq!(int_str(Some(-100)), "-100");
    }
}
