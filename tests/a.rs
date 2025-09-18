#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

#[cfg(test)]
mod tests {
    use formatm::multi;

    use super::*;

    #[test]
    fn lol() {
        let a = #[multi]
        /// haha
        /// again
        (foo = 4, a = 100);

        assert_eq!(a, "haha")
    }
}
