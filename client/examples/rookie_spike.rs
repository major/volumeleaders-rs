use std::collections::BTreeSet;

use rookie::common::enums::Cookie;

const DOMAIN: &str = "volumeleaders.com";
const REQUIRED_COOKIES: [&str; 3] = [
    "ASP.NET_SessionId",
    ".ASPXAUTH",
    "__RequestVerificationToken",
];

fn main() {
    let domains = Some(vec![DOMAIN.to_owned()]);

    match rookie::chrome(domains.clone()) {
        Ok(cookies) => report_cookies(&cookies),
        Err(chrome_err) => {
            println!("Chrome cookie extraction failed: {}", chrome_err);
            match rookie::firefox(domains) {
                Ok(cookies) => report_cookies(&cookies),
                Err(firefox_err) => {
                    println!("{}", firefox_err);
                    println!("FALLBACK NEEDED: document manual cookie extraction");
                }
            }
        }
    }
}

fn report_cookies(cookies: &[Cookie]) {
    let names: BTreeSet<&str> = cookies.iter().map(|cookie| cookie.name.as_str()).collect();
    let names_found: Vec<&str> = REQUIRED_COOKIES
        .iter()
        .copied()
        .filter(|name| names.contains(name))
        .collect();

    println!("Cookie names found: {:?}", names_found);

    for cookie_name in REQUIRED_COOKIES {
        println!(
            "{}: {}",
            cookie_name,
            if names.contains(cookie_name) {
                "FOUND"
            } else {
                "MISSING"
            }
        );
    }

    println!(
        "All required cookies present: {}",
        REQUIRED_COOKIES
            .iter()
            .all(|cookie_name| names.contains(cookie_name))
    );
}
