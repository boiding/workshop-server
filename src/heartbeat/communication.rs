use hyper::Uri;

pub enum Message {
    Check(Vec<(String, Uri)>),
}
