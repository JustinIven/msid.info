pub mod autodiscover;
pub mod credential_type;
pub mod graph;
pub mod http;
pub mod odc;
pub mod openid;
pub mod userrealm;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// Everything outside the unreserved URL alphabet is escaped. Input is already validated against a
/// strict allowlist, so this is belt-and-braces against path injection.
const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'\\')
    .add(b':')
    .add(b'+')
    .add(b'&')
    .add(b'=')
    .add(b'\'');

pub fn encode_segment(value: &str) -> String {
    utf8_percent_encode(value, PATH_SEGMENT).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_url_significant_characters() {
        assert_eq!(encode_segment("user@contoso.com"), "user@contoso.com");
        assert_eq!(encode_segment("a+b@c.com"), "a%2Bb@c.com");
        assert_eq!(encode_segment("a/../b"), "a%2F..%2Fb");
    }
}
