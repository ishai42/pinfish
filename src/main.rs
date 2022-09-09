mod xdr;
mod oncrpc;

fn main() {
    use bytes::{Buf, BufMut};

    let mut buf = b"hello world"[..].take(5);
    let mut dst = vec![];

    dst.put(&mut buf);
    assert_eq!(dst, b"hello");

    let mut buf = buf.into_inner();
    dst.clear();
    dst.put(&mut buf);
    assert_eq!(dst, b" world");
}

/*
#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
*/
