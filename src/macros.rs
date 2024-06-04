#[macro_export]
macro_rules! if_let_map {
    ($i:ident to $p:pat => $r:expr) => {
        if let $p = $i {
            Some($r)
        } else {
            None
        }
    };
}
