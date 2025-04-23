use http::Request;

pub fn parse_request(raw_req: &str) -> Request<()> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    match req.parse(raw_req.as_bytes()) {
        Ok(status) => {
            if !status.is_complete() {
                panic!("http request parsing incomplete")
            }
        }
        Err(_) => {
            panic!("failed to parse http request")
        }
    }
    

    return Request::new(());
}
