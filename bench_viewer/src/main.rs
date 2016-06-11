#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate serde_json;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate clap;

use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use iron::prelude::*;
use iron::status::Status;
use staticfile::Static;
use mount::Mount;

use clap::{Arg, App};

mod benchmark;

fn main() {
    let matches = App::new("bench_viewer")
        .about("Presents crypto-bench results in a web view.")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .default_value("3000")
            .takes_value(true))
        .arg(Arg::with_name("host")
            .short("h")
            .long("host")
            .value_name("HOST")
            .default_value("127.0.0.1")
            .takes_value(true))
        .arg(Arg::with_name("demo mode")
            .short("d")
            .long("demo")
            .help("Run with a fake data set. Use in place of --file."))
        .arg(Arg::with_name("log")
            .short("f")
            .long("file")
            .value_name("FILE")
            .takes_value(true)
            .required(true)
            .conflicts_with("demo mode"))
        .get_matches();

    let port : u16 = matches.value_of("port").unwrap().parse().unwrap();
    let host = matches.value_of("host").unwrap();

    let mut mount = Mount::new();

    // Serve the static site at /
    mount.mount("/", Static::new(Path::new("web_root")));

    // ... but override the data file to be served up dynamically
    let data_handler = if matches.is_present("demo mode") {
        DataHandler::Demo
    } else {
        DataHandler::Live(
            Path::new(matches.value_of("log").unwrap()).to_owned()
        )
    };
    mount.mount("/data/data.js", data_handler);

    let listen = Iron::new(mount).http((host, port)).unwrap();
    println!("View results at http://{}:{}/", host, port);
    drop(listen);
}

fn parse_bench(logs: &str) -> Result<benchmark::Run, String> {
    let mut parser = benchmark::Parser::new();
    for line in logs.lines() {
        try!(parser.parse_line(line));
    }
    parser.complete()
}

enum DataHandler {
    Demo,
    Live(PathBuf)
}

impl iron::middleware::Handler for DataHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        match *self {
            DataHandler::Demo => DataHandler::data_handler(DEMO_DATA),
            DataHandler::Live(ref path) => {
                let mut f = try!(File::open(path).map_err(|err| {
                    IronError::new(err, (
                        Status::InternalServerError,
                        "Benchmark log file not available on the server"
                    ))
                }));
                let mut s = String::new();
                try!(f.read_to_string(&mut s).map_err(|err| {
                    IronError::new(err, (
                        Status::InternalServerError,
                        "Couldn't read the log file"
                    ))
                }));
                DataHandler::data_handler(&s)
            }
        }
    }
}

impl DataHandler {
    fn data_handler(logs: &str) -> IronResult<Response> {
        use iron::headers::ContentType;
        // TODO: benchmark error handling
        let run = parse_bench(logs).unwrap();
        let data = serde_json::to_string(&run).unwrap();
        let mut response = Response::with((iron::status::Ok, data));
        response.headers.set(ContentType::json());
        Ok(response)
    }
}

static DEMO_DATA: &'static str = "
Running 'bench' in fastpbkdf2
Running 'bench' in octavo

running 24 tests
test digest::sha1::_1000       ... bench:       2,175 ns/iter (+/- 186) = 459 MB/s
test digest::sha1::_16         ... bench:         172 ns/iter (+/- 58) = 93 MB/s
test digest::sha1::_2000       ... bench:       4,326 ns/iter (+/- 1,494) = 462 MB/s
test digest::sha1::_256        ... bench:         718 ns/iter (+/- 217) = 356 MB/s
test digest::sha1::_8192       ... bench:      20,050 ns/iter (+/- 5,033) = 408 MB/s
test digest::sha1::block_len   ... bench:         320 ns/iter (+/- 114) = 200 MB/s
test digest::sha256::_1000     ... bench:       5,410 ns/iter (+/- 1,894) = 184 MB/s
test digest::sha256::_16       ... bench:         419 ns/iter (+/- 222) = 38 MB/s
test digest::sha256::_2000     ... bench:      10,781 ns/iter (+/- 3,839) = 185 MB/s
test digest::sha256::_256      ... bench:       1,780 ns/iter (+/- 423) = 143 MB/s
test digest::sha256::_8192     ... bench:      43,231 ns/iter (+/- 13,558) = 189 MB/s
test digest::sha256::block_len ... bench:         752 ns/iter (+/- 276) = 85 MB/s
test digest::sha384::_1000     ... bench:       3,067 ns/iter (+/- 983) = 326 MB/s
test digest::sha384::_16       ... bench:         425 ns/iter (+/- 177) = 37 MB/s
test digest::sha384::_2000     ... bench:       5,993 ns/iter (+/- 351) = 333 MB/s
test digest::sha384::_256      ... bench:       1,459 ns/iter (+/- 518) = 175 MB/s
test digest::sha384::_8192     ... bench:      25,250 ns/iter (+/- 5,992) = 324 MB/s
test digest::sha384::block_len ... bench:         911 ns/iter (+/- 425) = 140 MB/s
test digest::sha512::_1000     ... bench:       3,088 ns/iter (+/- 829) = 323 MB/s
test digest::sha512::_16       ... bench:         492 ns/iter (+/- 243) = 32 MB/s
test digest::sha512::_2000     ... bench:       6,790 ns/iter (+/- 2,054) = 294 MB/s
test digest::sha512::_256      ... bench:       1,381 ns/iter (+/- 344) = 185 MB/s
test digest::sha512::_8192     ... bench:      23,921 ns/iter (+/- 2,311) = 342 MB/s
test digest::sha512::block_len ... bench:         799 ns/iter (+/- 7) = 160 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 24 measured

Running 'bench' in openssl

running 25 tests
test digest::sha1::_1000       ... bench:       1,875 ns/iter (+/- 468) = 533 MB/s
test digest::sha1::_16         ... bench:         409 ns/iter (+/- 77) = 39 MB/s
test digest::sha1::_2000       ... bench:       2,860 ns/iter (+/- 178) = 699 MB/s
test digest::sha1::_256        ... bench:         713 ns/iter (+/- 203) = 359 MB/s
test digest::sha1::_8192       ... bench:      10,502 ns/iter (+/- 1,624) = 780 MB/s
test digest::sha1::block_len   ... bench:         461 ns/iter (+/- 193) = 138 MB/s
test digest::sha256::_1000     ... bench:       3,175 ns/iter (+/- 107) = 314 MB/s
test digest::sha256::_16       ... bench:         536 ns/iter (+/- 226) = 29 MB/s
test digest::sha256::_2000     ... bench:       6,011 ns/iter (+/- 1,530) = 332 MB/s
test digest::sha256::_256      ... bench:       1,202 ns/iter (+/- 536) = 212 MB/s
test digest::sha256::_8192     ... bench:      23,088 ns/iter (+/- 5,206) = 354 MB/s
test digest::sha256::block_len ... bench:         709 ns/iter (+/- 257) = 90 MB/s
test digest::sha384::_1000     ... bench:       2,464 ns/iter (+/- 75) = 405 MB/s
test digest::sha384::_16       ... bench:         659 ns/iter (+/- 270) = 24 MB/s
test digest::sha384::_2000     ... bench:       4,514 ns/iter (+/- 1,836) = 443 MB/s
test digest::sha384::_256      ... bench:       1,148 ns/iter (+/- 425) = 222 MB/s
test digest::sha384::_8192     ... bench:      16,764 ns/iter (+/- 2,781) = 488 MB/s
test digest::sha384::block_len ... bench:         937 ns/iter (+/- 576) = 136 MB/s
test digest::sha512::_1000     ... bench:       2,462 ns/iter (+/- 36) = 406 MB/s
test digest::sha512::_16       ... bench:         687 ns/iter (+/- 186) = 23 MB/s
test digest::sha512::_2000     ... bench:       4,463 ns/iter (+/- 1,481) = 448 MB/s
test digest::sha512::_256      ... bench:       1,171 ns/iter (+/- 465) = 218 MB/s
test digest::sha512::_8192     ... bench:      16,883 ns/iter (+/- 2,598) = 485 MB/s
test digest::sha512::block_len ... bench:         921 ns/iter (+/- 285) = 138 MB/s
test pbkdf2::hmac_sha1         ... bench:  68,457,057 ns/iter (+/- 4,447,843)

test result: ok. 0 passed; 0 failed; 0 ignored; 25 measured

Running 'bench' in ring

running 65 tests
test aead::seal_in_place::aes_128_gcm::tls12_1350               ... bench:       1,098 ns/iter (+/- 237) = 1229 MB/s
test aead::seal_in_place::aes_128_gcm::tls12_16                 ... bench:         153 ns/iter (+/- 41) = 104 MB/s
test aead::seal_in_place::aes_128_gcm::tls12_8192               ... bench:       5,933 ns/iter (+/- 1,947) = 1380 MB/s
test aead::seal_in_place::aes_128_gcm::tls12_finished           ... bench:         164 ns/iter (+/- 46) = 73 MB/s
test aead::seal_in_place::aes_128_gcm::tls13_1350               ... bench:       1,093 ns/iter (+/- 309) = 1235 MB/s
test aead::seal_in_place::aes_128_gcm::tls13_8192               ... bench:       5,823 ns/iter (+/- 2,103) = 1406 MB/s
test aead::seal_in_place::aes_128_gcm::tls13_finished           ... bench:         147 ns/iter (+/- 18) = 217 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_1350               ... bench:       1,270 ns/iter (+/- 542) = 1062 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_16                 ... bench:         170 ns/iter (+/- 5) = 94 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_8192               ... bench:       6,573 ns/iter (+/- 1,922) = 1246 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_finished           ... bench:         180 ns/iter (+/- 72) = 66 MB/s
test aead::seal_in_place::aes_256_gcm::tls13_1350               ... bench:       1,219 ns/iter (+/- 57) = 1107 MB/s
test aead::seal_in_place::aes_256_gcm::tls13_8192               ... bench:       6,513 ns/iter (+/- 1,791) = 1257 MB/s
test aead::seal_in_place::aes_256_gcm::tls13_finished           ... bench:         157 ns/iter (+/- 51) = 203 MB/s
test aead::seal_in_place::chacha20_poly1305::tls12_1350         ... bench:       1,794 ns/iter (+/- 19) = 752 MB/s
test aead::seal_in_place::chacha20_poly1305::tls12_16           ... bench:         385 ns/iter (+/- 168) = 41 MB/s
test aead::seal_in_place::chacha20_poly1305::tls12_8192         ... bench:       8,141 ns/iter (+/- 3,603) = 1006 MB/s
test aead::seal_in_place::chacha20_poly1305::tls12_finished     ... bench:         417 ns/iter (+/- 49) = 28 MB/s
test aead::seal_in_place::chacha20_poly1305::tls13_1350         ... bench:       1,756 ns/iter (+/- 653) = 768 MB/s
test aead::seal_in_place::chacha20_poly1305::tls13_8192         ... bench:       8,055 ns/iter (+/- 2,660) = 1017 MB/s
test aead::seal_in_place::chacha20_poly1305::tls13_finished     ... bench:         366 ns/iter (+/- 159) = 87 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_1350     ... bench:       1,783 ns/iter (+/- 407) = 757 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_16       ... bench:         387 ns/iter (+/- 109) = 41 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_8192     ... bench:       8,166 ns/iter (+/- 3,976) = 1003 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_finished ... bench:         484 ns/iter (+/- 79) = 24 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls13_1350     ... bench:       2,127 ns/iter (+/- 638) = 634 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls13_8192     ... bench:       8,141 ns/iter (+/- 1,237) = 1006 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls13_finished ... bench:         378 ns/iter (+/- 48) = 84 MB/s
test agreement::p256::generate_key_pair                         ... bench:      22,613 ns/iter (+/- 10,662)
test agreement::p256::generate_key_pair_and_agree_ephemeral     ... bench:      84,529 ns/iter (+/- 13,288)
test agreement::p256::generate_private_key                      ... bench:       2,941 ns/iter (+/- 846)
test agreement::p384::generate_key_pair                         ... bench:     744,733 ns/iter (+/- 168,580)
test agreement::p384::generate_key_pair_and_agree_ephemeral     ... bench:     758,121 ns/iter (+/- 118,455)
test agreement::p384::generate_private_key                      ... bench:       3,977 ns/iter (+/- 960)
test agreement::x25519::generate_key_pair                       ... bench:      53,214 ns/iter (+/- 25,414)
test agreement::x25519::generate_key_pair_and_agree_ephemeral   ... bench:      52,421 ns/iter (+/- 21,623)
test agreement::x25519::generate_private_key                    ... bench:       2,440 ns/iter (+/- 627)
test digest::sha1::_1000                                        ... bench:       4,821 ns/iter (+/- 352) = 207 MB/s
test digest::sha1::_16                                          ... bench:         348 ns/iter (+/- 164) = 45 MB/s
test digest::sha1::_2000                                        ... bench:       9,518 ns/iter (+/- 3,060) = 210 MB/s
test digest::sha1::_256                                         ... bench:       1,551 ns/iter (+/- 175) = 165 MB/s
test digest::sha1::_8192                                        ... bench:      38,211 ns/iter (+/- 14,306) = 214 MB/s
test digest::sha1::block_len                                    ... bench:         657 ns/iter (+/- 86) = 97 MB/s
test digest::sha256::_1000                                      ... bench:       2,874 ns/iter (+/- 1,321) = 347 MB/s
test digest::sha256::_16                                        ... bench:         228 ns/iter (+/- 100) = 70 MB/s
test digest::sha256::_2000                                      ... bench:       5,702 ns/iter (+/- 1,595) = 350 MB/s
test digest::sha256::_256                                       ... bench:         946 ns/iter (+/- 374) = 270 MB/s
test digest::sha256::_8192                                      ... bench:      22,745 ns/iter (+/- 616) = 360 MB/s
test digest::sha256::block_len                                  ... bench:         420 ns/iter (+/- 177) = 152 MB/s
test digest::sha384::_1000                                      ... bench:       2,056 ns/iter (+/- 348) = 486 MB/s
test digest::sha384::_16                                        ... bench:         303 ns/iter (+/- 148) = 52 MB/s
test digest::sha384::_2000                                      ... bench:       4,048 ns/iter (+/- 1,374) = 494 MB/s
test digest::sha384::_256                                       ... bench:         802 ns/iter (+/- 193) = 319 MB/s
test digest::sha384::_8192                                      ... bench:      16,143 ns/iter (+/- 5,753) = 507 MB/s
test digest::sha384::block_len                                  ... bench:         557 ns/iter (+/- 108) = 229 MB/s
test digest::sha512::_1000                                      ... bench:       2,058 ns/iter (+/- 331) = 485 MB/s
test digest::sha512::_16                                        ... bench:         298 ns/iter (+/- 99) = 53 MB/s
test digest::sha512::_2000                                      ... bench:       4,040 ns/iter (+/- 1,151) = 495 MB/s
test digest::sha512::_256                                       ... bench:         805 ns/iter (+/- 277) = 318 MB/s
test digest::sha512::_8192                                      ... bench:      16,145 ns/iter (+/- 5,612) = 507 MB/s
test digest::sha512::block_len                                  ... bench:         559 ns/iter (+/- 170) = 228 MB/s
test pbkdf2::hmac_sha256                                        ... bench:  54,747,125 ns/iter (+/- 4,559,209)
test pbkdf2::hmac_sha512                                        ... bench:  75,506,846 ns/iter (+/- 17,366,296)
test signature::ed25519::generate_key_pair                      ... bench:      50,340 ns/iter (+/- 1,171)
test signature::ed25519::sign_empty                             ... bench:      48,658 ns/iter (+/- 646)

test result: ok. 0 passed; 0 failed; 0 ignored; 65 measured

Running 'bench' in rust_crypto

running 53 tests
test aead::seal_in_place::aes_128_gcm::tls12_1350               ... bench:      21,048 ns/iter (+/- 874) = 64 MB/s
test aead::seal_in_place::aes_128_gcm::tls12_16                 ... bench:       2,046 ns/iter (+/- 60) = 7 MB/s
test aead::seal_in_place::aes_128_gcm::tls12_8192               ... bench:     118,123 ns/iter (+/- 63,880) = 69 MB/s
test aead::seal_in_place::aes_128_gcm::tls12_finished           ... bench:       2,470 ns/iter (+/- 694) = 4 MB/s
test aead::seal_in_place::aes_128_gcm::tls13_1350               ... bench:      20,864 ns/iter (+/- 2,162) = 64 MB/s
test aead::seal_in_place::aes_128_gcm::tls13_8192               ... bench:     117,656 ns/iter (+/- 5,894) = 69 MB/s
test aead::seal_in_place::aes_128_gcm::tls13_finished           ... bench:       2,272 ns/iter (+/- 55) = 14 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_1350               ... bench:      21,859 ns/iter (+/- 876) = 61 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_16                 ... bench:       2,094 ns/iter (+/- 38) = 7 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_8192               ... bench:     122,314 ns/iter (+/- 3,618) = 66 MB/s
test aead::seal_in_place::aes_256_gcm::tls12_finished           ... bench:       2,094 ns/iter (+/- 258) = 5 MB/s
test aead::seal_in_place::aes_256_gcm::tls13_1350               ... bench:      21,696 ns/iter (+/- 6,396) = 62 MB/s
test aead::seal_in_place::aes_256_gcm::tls13_8192               ... bench:     122,170 ns/iter (+/- 3,335) = 67 MB/s
test aead::seal_in_place::aes_256_gcm::tls13_finished           ... bench:       2,337 ns/iter (+/- 764) = 13 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_1350     ... bench:       7,152 ns/iter (+/- 3,701) = 188 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_16       ... bench:         527 ns/iter (+/- 125) = 30 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_8192     ... bench:      33,104 ns/iter (+/- 5,331) = 247 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls12_finished ... bench:         453 ns/iter (+/- 69) = 26 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls13_1350     ... bench:       5,766 ns/iter (+/- 1,504) = 234 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls13_8192     ... bench:      32,896 ns/iter (+/- 13,081) = 249 MB/s
test aead::seal_in_place::chacha20_poly1305_old::tls13_finished ... bench:         474 ns/iter (+/- 9) = 67 MB/s
test agreement::x25519::generate_key_pair                       ... bench:     141,818 ns/iter (+/- 20,878)
test agreement::x25519::generate_key_pair_and_agree_ephemeral   ... bench:     283,582 ns/iter (+/- 74,218)
test agreement::x25519::generate_private_key                    ... bench:       2,432 ns/iter (+/- 350)
test digest::sha1::_1000                                        ... bench:       1,997 ns/iter (+/- 754) = 500 MB/s
test digest::sha1::_16                                          ... bench:         156 ns/iter (+/- 48) = 102 MB/s
test digest::sha1::_2000                                        ... bench:       3,971 ns/iter (+/- 1,382) = 503 MB/s
test digest::sha1::_256                                         ... bench:         642 ns/iter (+/- 25) = 398 MB/s
test digest::sha1::_8192                                        ... bench:      15,808 ns/iter (+/- 6,805) = 518 MB/s
test digest::sha1::block_len                                    ... bench:         275 ns/iter (+/- 135) = 232 MB/s
test digest::sha256::_1000                                      ... bench:       4,449 ns/iter (+/- 74) = 224 MB/s
test digest::sha256::_16                                        ... bench:         310 ns/iter (+/- 134) = 51 MB/s
test digest::sha256::_2000                                      ... bench:       8,976 ns/iter (+/- 3,780) = 222 MB/s
test digest::sha256::_256                                       ... bench:       1,421 ns/iter (+/- 549) = 180 MB/s
test digest::sha256::_8192                                      ... bench:      35,775 ns/iter (+/- 14,269) = 228 MB/s
test digest::sha256::block_len                                  ... bench:         588 ns/iter (+/- 99) = 108 MB/s
test digest::sha384::_1000                                      ... bench:       2,844 ns/iter (+/- 1,003) = 351 MB/s
test digest::sha384::_16                                        ... bench:         381 ns/iter (+/- 175) = 41 MB/s
test digest::sha384::_2000                                      ... bench:       5,658 ns/iter (+/- 974) = 353 MB/s
test digest::sha384::_256                                       ... bench:       1,087 ns/iter (+/- 371) = 235 MB/s
test digest::sha384::_8192                                      ... bench:      22,845 ns/iter (+/- 6,663) = 358 MB/s
test digest::sha384::block_len                                  ... bench:         733 ns/iter (+/- 220) = 174 MB/s
test digest::sha512::_1000                                      ... bench:       2,847 ns/iter (+/- 890) = 351 MB/s
test digest::sha512::_16                                        ... bench:         385 ns/iter (+/- 137) = 41 MB/s
test digest::sha512::_2000                                      ... bench:       5,660 ns/iter (+/- 213) = 353 MB/s
test digest::sha512::_256                                       ... bench:       1,090 ns/iter (+/- 359) = 234 MB/s
test digest::sha512::_8192                                      ... bench:      22,832 ns/iter (+/- 563) = 358 MB/s
test digest::sha512::block_len                                  ... bench:         738 ns/iter (+/- 330) = 173 MB/s
test pbkdf2::hmac_sha1                                          ... bench:  60,563,177 ns/iter (+/- 13,445,380)
test pbkdf2::hmac_sha256                                        ... bench: 142,183,589 ns/iter (+/- 29,910,882)
test pbkdf2::hmac_sha512                                        ... bench: 167,192,029 ns/iter (+/- 31,000,862)
test signature::ed25519::generate_key_pair                      ... bench:      52,149 ns/iter (+/- 10,549)
test signature::ed25519::sign_empty                             ... bench:      50,918 ns/iter (+/- 14,703)

test result: ok. 0 passed; 0 failed; 0 ignored; 53 measured

Running 'bench' in sodiumoxide

running 15 tests
test aead::xsalsa20poly1305_1350 ... bench:       6,504 ns/iter (+/- 2,668) = 207 MB/s
test aead::xsalsa20poly1305_16   ... bench:         495 ns/iter (+/- 160) = 32 MB/s
test aead::xsalsa20poly1305_8192 ... bench:      29,689 ns/iter (+/- 3,375) = 275 MB/s
test digest::sha256::_1000       ... bench:       5,613 ns/iter (+/- 1,726) = 178 MB/s
test digest::sha256::_16         ... bench:         397 ns/iter (+/- 132) = 40 MB/s
test digest::sha256::_2000       ... bench:      11,229 ns/iter (+/- 4,299) = 178 MB/s
test digest::sha256::_256        ... bench:       1,805 ns/iter (+/- 303) = 141 MB/s
test digest::sha256::_8192       ... bench:      45,102 ns/iter (+/- 18,494) = 181 MB/s
test digest::sha256::block_len   ... bench:         753 ns/iter (+/- 284) = 84 MB/s
test digest::sha512::_1000       ... bench:       3,575 ns/iter (+/- 1,755) = 279 MB/s
test digest::sha512::_16         ... bench:         493 ns/iter (+/- 18) = 32 MB/s
test digest::sha512::_2000       ... bench:       6,945 ns/iter (+/- 753) = 287 MB/s
test digest::sha512::_256        ... bench:       1,370 ns/iter (+/- 483) = 186 MB/s
test digest::sha512::_8192       ... bench:      28,132 ns/iter (+/- 8,035) = 291 MB/s
test digest::sha512::block_len   ... bench:         930 ns/iter (+/- 175) = 137 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 15 measured
";
