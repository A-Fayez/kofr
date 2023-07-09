use std::env;

fn main() {
    let uri = env::var("CONNECT_URI").expect("env var CONNECT_URI not found");

    let body = reqwest::blocking::get(uri)
        .unwrap()
        .text()
        .unwrap();
    println!("body = {:?}", body);
}
