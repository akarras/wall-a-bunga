/// Returns a number formatted with a suffix of k, or m.
/// Will return a decimal point if applicable
pub(crate) fn trendy_number_format(val: f64) -> String {
    let log = val.log10();
    let val_suff = if (3.0..6.0).contains(&log) {
        Some((val / 1000.0, "k"))
    } else if (6.0..9.0).contains(&log) {
        Some((val / 1000000.0, "m"))
    } else if log >= 9.0 {
        Some((val / 1000000000.0, "b"))
    } else {
        None
    };
    match val_suff {
        // no suffix
        None => format!("{:.0}", val),
        // suffix, use the applied units
        Some((sig, suffix)) => format!("{:.1}{}", sig, suffix),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn num_format_test() {
        assert_eq!(trendy_number_format(12300u32 as f64), "12.3k");
        assert_eq!(trendy_number_format(12012u32 as f64), "12.0k");
        assert_eq!(trendy_number_format(1200000u32 as f64), "1.2m");
        assert_eq!(trendy_number_format(10001u32 as f64), "10.0k");
        assert_eq!(trendy_number_format(1u32 as f64), "1");
    }
}
