fn main() {
    docstr::docstr!(format
        /// hello {world}
    );

    docstr::docstr!(std format
        /// hello {world}
    );

    docstr::docstr!(std format!
        /// hello {world}
    );

    docstr::docstr!(std:::format!
        /// hello {world}
    );

    docstr::docstr!(std:format!
        /// hello {world}
    );

    docstr::docstr!(::std::format::!
        /// hello {world}
    );
}
