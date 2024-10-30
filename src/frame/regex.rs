

#[macro_export]
macro_rules! lazy_static_regex {
    ($name:ident, $regex:expr) => {
        static ONCE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

        fn $name() -> &'static regex::Regex {
            ONCE.get_or_init(|| regex::Regex::new($regex).expect("Invalid regex"))
        }
    };
}

lazy_static_regex!(
    address_regex,
    r"^(X|Y|F|B|M|L|SM|SD|D|R|ZR|W|TN|TS|CN|CS)(.+)$"
);

pub fn split_address(address: &str) -> Option<(&str, &str)> {
    let regex = address_regex();

    match regex.captures(address) {
        Some(caps) => {
            let address_prefix = caps.get(1).unwrap().as_str();
            let address_number = caps.get(2).unwrap().as_str();
            Some((address_prefix, address_number))
        }
        None => None,
    }
}