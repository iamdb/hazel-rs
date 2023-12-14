fn main() {
    pkg_config::Config::new()
        .statik(true)
        .atleast_version("1.2.13")
        .probe("zlib")
        .expect("error linking lib");

    pkg_config::Config::new()
        .statik(true)
        .atleast_version("0.4.41")
        .probe("libzen")
        .expect("error linking lib");

    pkg_config::Config::new()
        .statik(true)
        .atleast_version("23.06")
        .probe("libmediainfo")
        .expect("error linking lib");
}
