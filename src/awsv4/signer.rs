use super::date::{Date, DateFormatting as _};

type SecretKey = str;
type Region = str;

fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut state = sha2::Sha256::new();

    state.update(data);
    state.finalize().into()
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    use hmac::Mac;
    type HmacSha256 = hmac::Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(message);

    mac.finalize().into_bytes().into()
}

fn scope(date: &Date, region: &Region) -> String {
    format!(
        "{}/{}/s3/aws4_request",
        date.format_yyyymmddthhmmssz(),
        region
    )
}

fn string_to_sign(
    // datestring: &str,
    date: &Date,
    canonical_request: &str,
    region: &Region,
) -> String {
    let algorithm = "AWS4-HMAC-SHA256";
    format!(
        "{}\n{}\n{}\n{}",
        algorithm,
        date.format_yyyymmddthhmmssz(),
        scope(date, region),
        hex::encode(sha256(canonical_request.as_bytes()))
    )
}

fn canonical_request(
    http_method: &str,
    canonical_uri: &str,
    canonical_query_string: &str,
    canonical_headers: &str,
    signed_headers: &str,
    hashed_payload: &str,
) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        http_method,
        canonical_uri,
        canonical_query_string,
        canonical_headers,
        signed_headers,
        hashed_payload
    )
}

pub fn compute_signature(
    secret_key: &SecretKey,
    date: &Date,
    region: &Region,
    string_to_sign: &str,
) -> String {
    let datekey = hmac_sha256(
        format!("AWS{}", secret_key).as_bytes(),
        date.format_yyyymmddthhmmssz().as_bytes(),
    );
    let regionkey = hmac_sha256(&datekey, region.as_bytes());
    let servicekey = hmac_sha256(&regionkey, "s3".as_bytes());
    let signingkey = hmac_sha256(&servicekey, "aws4_request".as_bytes());

    hex::encode(hmac_sha256(&signingkey, string_to_sign.as_bytes()))
}



#[derive(Debug)]
enum AwsSignatureV4Error {}

#[test]
fn test_fn() {
    let date: Date = chrono::Utc::now();

    println!("{:?}", date.format_yyyymmddthhmmssz());
}
