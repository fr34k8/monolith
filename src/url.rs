use base64::{prelude::BASE64_STANDARD, Engine};
use percent_encoding::percent_decode_str;
pub use url::Url;

use crate::core::{detect_media_type, parse_content_type};

pub const EMPTY_IMAGE_DATA_URL: &str = "data:image/png,\
%89PNG%0D%0A%1A%0A%00%00%00%0DIHDR%00%00%00%0D%00%00%00%0D%08%04%00%00%00%D8%E2%2C%F7%00%00%00%11IDATx%DAcd%C0%09%18G%A5%28%96%02%00%0A%F8%00%0E%CB%8A%EB%16%00%00%00%00IEND%AEB%60%82";

pub fn clean_url(url: Url) -> Url {
    let mut url = url.clone();

    // Clear fragment (if any)
    url.set_fragment(None);

    url
}

pub fn create_data_url(media_type: &str, charset: &str, data: &[u8], final_asset_url: &Url) -> Url {
    // TODO: move this block out of this function
    let media_type: String = if media_type.is_empty() {
        detect_media_type(data, final_asset_url)
    } else {
        media_type.to_string()
    };

    let mut data_url: Url = Url::parse("data:,").unwrap();

    let c: String =
        if !charset.trim().is_empty() && !charset.trim().eq_ignore_ascii_case("US-ASCII") {
            format!(";charset={}", charset.trim())
        } else {
            "".to_string()
        };

    data_url.set_path(
        format!(
            "{}{};base64,{}",
            media_type,
            c,
            BASE64_STANDARD.encode(data)
        )
        .as_str(),
    );

    data_url
}

pub fn domain_is_within_domain(domain: &str, domain_to_match_against: &str) -> bool {
    if domain_to_match_against.is_empty() {
        return false;
    }

    if domain_to_match_against == "." {
        return true;
    }

    let domain_partials: Vec<&str> = domain.trim_end_matches(".").rsplit(".").collect();
    let domain_to_match_against_partials: Vec<&str> = domain_to_match_against
        .trim_end_matches(".")
        .rsplit(".")
        .collect();
    let domain_to_match_against_starts_with_a_dot = domain_to_match_against.starts_with(".");

    let mut i: usize = 0;
    let l: usize = std::cmp::max(
        domain_partials.len(),
        domain_to_match_against_partials.len(),
    );
    let mut ok: bool = true;

    while i < l {
        // Exit and return false if went out of bounds of domain to match against, and it didn't start with a dot
        if !domain_to_match_against_starts_with_a_dot
            && domain_to_match_against_partials.len() < i + 1
        {
            ok = false;
            break;
        }

        let domain_partial = if domain_partials.len() < i + 1 {
            ""
        } else {
            domain_partials.get(i).unwrap()
        };
        let domain_to_match_against_partial = if domain_to_match_against_partials.len() < i + 1 {
            ""
        } else {
            domain_to_match_against_partials.get(i).unwrap()
        };

        let parts_match = domain_to_match_against_partial.eq_ignore_ascii_case(domain_partial);

        if !parts_match && !domain_to_match_against_partial.is_empty() {
            ok = false;
            break;
        }

        i += 1;
    }

    ok
}

pub fn is_url_and_has_protocol(input: &str) -> bool {
    match Url::parse(input) {
        Ok(parsed_url) => !parsed_url.scheme().is_empty(),
        Err(_) => false,
    }
}

pub fn parse_data_url(url: &Url) -> (String, String, Vec<u8>) {
    let path: String = url.path().to_string();
    let comma_loc: usize = path.find(',').unwrap_or(path.len());

    // Split data URL into meta data and raw data
    let content_type: String = path.chars().take(comma_loc).collect();
    let data: String = path.chars().skip(comma_loc + 1).collect();

    // Parse meta data
    let (media_type, charset, is_base64) = parse_content_type(&content_type);

    // Parse raw data into vector of bytes
    let text: String = percent_decode_str(&data).decode_utf8_lossy().to_string();
    let blob: Vec<u8> = if is_base64 {
        BASE64_STANDARD.decode(&text).unwrap_or_default()
    } else {
        text.as_bytes().to_vec()
    };

    (media_type, charset, blob)
}

pub fn get_referer_url(url: Url) -> Url {
    let mut url = url.clone();
    // Spec: https://httpwg.org/specs/rfc9110.html#field.referer
    // Must not include the fragment and userinfo components of the URI
    url.set_fragment(None);
    url.set_username("").unwrap();
    url.set_password(None).unwrap();

    url
}

pub fn resolve_url(from: &Url, to: &str) -> Url {
    match Url::parse(to) {
        Ok(parsed_url) => parsed_url,
        Err(_) => match from.join(to) {
            Ok(joined) => joined,
            Err(_) => Url::parse("data:,").unwrap(),
        },
    }
}
