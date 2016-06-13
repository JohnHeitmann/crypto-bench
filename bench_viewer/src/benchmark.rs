/// Terminology (probably terrible):
/// An `Item` is an an individual test run (ie a thing marked #[bench])
/// A `Suite` is a benchmark run for a crate (ie what cargo bench runs)
/// A `Run` is crypto-bench's umbrella run of multiple crates

use std::str;
use regex::Regex;
use regex::Captures;

#[derive(Serialize, Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub average_ns: i32,
    pub deviation_ns: i32,
    #[serde(skip_serializing_if="Option::is_none")]
    pub throughput_mbps: Option<i32>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Suite {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Run {
    pub architecture: String,
    pub suites: Vec<Suite>,
}

pub struct Parser {
    run: Run,
}

lazy_static! {
    static ref SUITE_REGEX: Regex = Regex::new(r"^Running 'bench' in (?P<name>[^']*)$").unwrap();

    // A result item looks like this:
    // test digest::sha1::_1000       ... bench:       2,188 ns/iter (+/- 205) = 457 MB/s
    // or this (no MB/s):
    // test pbkdf2::hmac_sha1          ... bench: 60,563,177 ns/iter (+/- 13,445,380)
    static ref ITEM_REGEX: Regex = Regex::new(concat!(
        // test digest::sha1::_1000     ... bench:
        r"^test (?P<name>[^\s]+)\s*... bench:",
        // 2,188 ns/iter
        // 'ns/iter' is hardcoded in libtest. The unit will not vary
        r"\s*(?P<average>[^\s]+) ns/iter ",
        // (+/- 205)
        r"\(\+/- (?P<deviation>[^\s]+)\)",
        // = 457 MB/s
        // Again, the unit is hardcoded in libtest and won't vary
        r"( = (?P<throughput>[^\s]+) MB/s)?"
    )).unwrap();
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            run: Run {
                architecture: "TODO".to_owned(),
                suites: vec![],
            }
        }
    }

    pub fn parse_line(&mut self, line: &str) -> Result<(), String> {
        if let Some(item_capture) = ITEM_REGEX.captures(line) {
            return self.parse_item(item_capture);
        } else if let Some(suite_capture) = SUITE_REGEX.captures(line) {
            return self.parse_suite(suite_capture);
        }
        Ok(())
    }

    fn parse_bench_num<T>(s: &str) -> Result<T, String>
        where T: str::FromStr {
        // libtest manually inserts commas. ie it doesn't
        // localize, so we don't have to either.
        let bare_s : String = s.chars().filter(|c| *c != ',').collect();
        match bare_s.parse::<T>() {
            Ok(i) => Ok(i),
            Err(_) => Err(format!("Couldn't parse a number from '{}'", s))
        }
    }

    fn parse_item(&mut self, capture: Captures) -> Result<(), String> {
        let item = Item {
            name: capture.name("name").unwrap().to_owned(),
            average_ns: try!(Parser::parse_bench_num(capture.name("average").unwrap())),
            deviation_ns: try!(Parser::parse_bench_num(capture.name("deviation").unwrap())),
            throughput_mbps: match capture.name("throughput") {
                Some(throughput) => Some(try!(Parser::parse_bench_num(throughput))),
                None => None
            },
        };
        match self.run.suites.last_mut() {
            Some(ref mut suite) => {
                suite.items.push(item);
                Ok(())
            },
            None => Err("Encountered a result without seeing a suite name first".to_string())
        }
    }

    fn parse_suite(&mut self, capture: Captures) -> Result<(), String> {
        let suite = Suite {
            name: capture.name("name").unwrap().to_owned(),
            items: vec![],
        };
        self.run.suites.push(suite);
        Ok(())
    }

    pub fn complete(self) -> Result<Run, String> {
        Ok(self.run)
    }
}

mod test {
    use super::*;

    #[test]
    fn smoketest() {
        let mut parser = Parser::new();
        for line in SMOKETEST_DATA.lines() {
            parser.parse_line(line).unwrap();
        }
        let result = parser.complete().unwrap();

        let expected = Run {
            architecture: "TODO".to_owned(),
            suites: vec![
                Suite {
                    name: "demo-area".to_owned(),
                    items: vec![
                        Item {
                            name: "digest::sha1::_1000".to_owned(),
                            average_ns: 2175,
                            deviation_ns: 186,
                            throughput_mbps: Some(459),
                        }
                    ],
                },
                Suite {
                    name: "demo-area2".to_owned(),
                    items: vec![
                        Item {
                            name: "agreement::p256::generate_key_pair".to_owned(),
                            average_ns: 22613,
                            deviation_ns: 10662,
                            throughput_mbps: None,
                        }
                    ],
                },
            ],
        };

        assert_eq!(expected, result);
    }

    static SMOKETEST_DATA: &'static str = "
Running 'bench' in demo-area
test digest::sha1::_1000       ... bench:       2,175 ns/iter (+/- 186) = 459 MB/s

test result: ok. 0 passed; 0 failed; 0 ignored; 25 measured

Running 'bench' in demo-area2
test agreement::p256::generate_key_pair                         ... bench:      22,613 ns/iter (+/- 10,662)
";

}


// TODO smoke testing.
// Test skipped benchmarks like AES on disabled hardware