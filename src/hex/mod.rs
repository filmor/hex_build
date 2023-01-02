mod names {
    include!(concat!(env!("OUT_DIR"), "/hex.names.rs"));
}
mod package {
    include!(concat!(env!("OUT_DIR"), "/hex.package.rs"));
}
mod signed {
    include!(concat!(env!("OUT_DIR"), "/hex.signed.rs"));
}
mod versions {
    include!(concat!(env!("OUT_DIR"), "/hex.versions.rs"));
}

mod repo;
mod config;
mod tarball;
mod consult;
