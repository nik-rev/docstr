fn main() {
    docstr::docstr!(
        #()
    );

    docstr::docstr!(
        #{}
    );

    docstr::docstr!(
        #[foo]
    );

    docstr::docstr!(
        #[doc ? ]
    );

    docstr::docstr!(
        #[doc = true]
    );

    docstr::docstr!(
        #[doc = 100]
    );

    docstr::docstr!(
        #[doc = b"byte string"]
    );
}
