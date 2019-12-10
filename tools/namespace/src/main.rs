#[macro_use]
extern crate clap;
extern crate sawtk;

use clap::{App, Arg};
use sawtk::ns;
use std::env;

fn main() {
    let matches = App::new("namespace tool")
        .version(crate_version!())
        .author("Kigi Chang <kigi.chang@gmail.com>")
        .args(&[
            Arg::with_name("name")
                .long("name")
                .short("n")
                .required(true)
                .help("name for namespace")
                .takes_value(true),
            Arg::with_name("input")
                .long("input")
                .short("i")
                .required(true)
                .help("key to compute address")
                .takes_value(true),
        ])
        .get_matches();

    let name = matches.value_of("name").unwrap();

    let namespace = ns::new(&name);
    println!("name: {}, prefix: {}", namespace.name(), namespace.prefix());

    let test = matches.value_of("input").unwrap();

    for x in test.split(",").collect::<Vec<_>>().iter() {
        println!("{}:{}", x, namespace.make_address(x));
    }
}
