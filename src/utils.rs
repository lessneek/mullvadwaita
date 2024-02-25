pub fn gettext(text: &'static str) -> &'static str {
    text
}

#[macro_export]
macro_rules! choose {
    ($c:expr, $v:expr, $v1:expr) => {
        if $c {
            $v
        } else {
            $v1
        }
    };
}
