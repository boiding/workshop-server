use hyper::Uri;

pub enum Message {
    Pick(Vec<(String, Uri, String)>),
}
