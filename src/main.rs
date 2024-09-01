fn main() {
    let current_version = env!("CARGO_PKG_VERSION");
    let res =
        reqwest::blocking::get("https://github.com/AEGEE-Delft/penning-helper/releases/latest");
    match res {
        Ok(res) => {
            let url_version = res.url().path().split_once("tag/v").unwrap().1;
            let uv_sv = semver::Version::parse(url_version).unwrap();
            let pkg_sv = semver::Version::parse(&current_version).unwrap();
            println!("Current {}, Latest {}, Up to date: {}", pkg_sv, uv_sv, pkg_sv >= uv_sv);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
