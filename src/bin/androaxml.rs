extern crate androrust;

use androrust::file;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let apk = match file::apk::open(&args[2]) {
        Ok(apk) => apk,
        Err(e) => panic!("error: {:?}", e)
    };
}