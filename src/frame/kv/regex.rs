use crate::lazy_static_regex;

lazy_static_regex!(
    address_regex,
    r"^(R|X|Y|B|DM|FM|MR|LR|CR|CM|EM|FM|ZF|T|C|M|L|D|F)(.+)$"
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