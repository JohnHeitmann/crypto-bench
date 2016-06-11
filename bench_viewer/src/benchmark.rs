/// Terminology (probably terrible):
/// An `Item` is an an individual test run (ie a thing marked #[bench])
/// A `Suite` is a benchmark run for a crate (ie what cargo bench runs)
/// A `Run` is crypto-bench's umbrella run of multiple crates

use std::str;
use regex::Regex;
use regex::Captures;

#[derive(Serialize, Debug)]
pub struct Item {
    pub name: String,
    pub average_ns: i32,
    pub deviation_ns: i32,
    pub throughput_mbps: f32,
}

#[derive(Serialize, Debug)]
pub struct Suite {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Serialize, Debug)]
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
    static ref ITEM_REGEX: Regex = Regex::new(concat!(
        // test digest::sha1::_1000     ... bench:
        r"^test (?P<name>[^\s]+)\s*... bench:",
        // 2,188 ns/iter
        // 'ns/iter' is hardcoded in libtest. The unit will not vary
        r"\s*(?P<average>[^\s]+) ns/iter ",
        // (+/- 205)
        r"\(\+/- (?P<deviation>[^\s]+)\) ",
        // = 457 MB/s
        // Again, the unit is hardcoded in libtest and won't vary
        r"= (?P<throughput>[^\s]+) MB/s"
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
            throughput_mbps: try!(Parser::parse_bench_num(capture.name("throughput").unwrap())),
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

// TODO smoke testing.
// Test skipped benchmarks like AES on disabled hardware