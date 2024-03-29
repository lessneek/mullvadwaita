#[allow(unused_imports)]
pub(crate) use log::{debug, error, info, trace, warn};

pub(crate) trait ToStr {
    fn to_str(&self) -> &str;
}

impl ToStr for Option<String> {
    fn to_str(self: &Option<String>) -> &str {
        self.as_ref().map(|ss| ss.as_str()).unwrap_or_default()
    }
}
