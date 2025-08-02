use super::types::DataOProcess;

// Optimization: use static array instead of HashMap for better lookup performance
const KV_INSTRUCTIONS: &[(&str, &str, DataOProcess)] = &[
    ("R", "X", DataOProcess::Hex),
    ("MR", "M", DataOProcess::Decimal),
    ("LR", "L", DataOProcess::Decimal),
    ("DM", "D", DataOProcess::None),
    ("FM", "R", DataOProcess::None),
    ("B", "B", DataOProcess::None),
    ("ZF", "ZR", DataOProcess::DecimalToHex),
    // XYM markers
    ("M", "M", DataOProcess::None),
    ("D", "D", DataOProcess::None),
    ("F", "R", DataOProcess::None),
    ("L", "L", DataOProcess::None),
    // Special
    ("X", "X", DataOProcess::XYToHex),
    ("Y", "Y", DataOProcess::XYToHex),
];

// Optimized lookup using linear search - faster for small arrays
// For 13 elements, linear search is typically faster than HashMap due to better cache locality
#[inline]
pub fn find(prefix: &str) -> Option<(&'static str, DataOProcess)> {
    // Fast path for single-character prefixes using byte comparison
    if prefix.len() == 1 {
        let prefix_byte = prefix.as_bytes()[0];
        for &(key, value, process) in KV_INSTRUCTIONS {
            if key.len() == 1 && key.as_bytes()[0] == prefix_byte {
                return Some((value, process));
            }
        }
    } else {
        // Two-character prefixes
        for &(key, value, process) in KV_INSTRUCTIONS {
            if key == prefix {
                return Some((value, process));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_single_char() {
        assert_eq!(find("R"), Some(("X", DataOProcess::Hex)));
        assert_eq!(find("M"), Some(("M", DataOProcess::None)));
        assert_eq!(find("D"), Some(("D", DataOProcess::None)));
        assert_eq!(find("X"), Some(("X", DataOProcess::XYToHex)));
        assert_eq!(find("Y"), Some(("Y", DataOProcess::XYToHex)));
    }

    #[test]
    fn test_find_two_char() {
        assert_eq!(find("MR"), Some(("M", DataOProcess::Decimal)));
        assert_eq!(find("LR"), Some(("L", DataOProcess::Decimal)));
        assert_eq!(find("DM"), Some(("D", DataOProcess::None)));
        assert_eq!(find("FM"), Some(("R", DataOProcess::None)));
        assert_eq!(find("ZF"), Some(("ZR", DataOProcess::DecimalToHex)));
    }

    #[test]
    fn test_find_not_found() {
        assert_eq!(find("Z"), None);
        assert_eq!(find("XX"), None);
        assert_eq!(find("ABC"), None);
    }
}
