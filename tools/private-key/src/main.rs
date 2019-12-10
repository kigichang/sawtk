#[macro_use]
extern crate clap;
extern crate sawtk;

use clap::{App, Arg, SubCommand};
use sawtk::signing;
use sawtk::util;
use sawtk::wallet;

use std::error::Error;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;


fn write_file(file_name: &str, data: &[u8]) -> Result<(), String> {
    let path = Path::new(file_name);
    let display = path.display();
    
    let mut file = File::create(&path).map_err(|e| format!("couldn't create {}: {}", display, e.description()))?;
    
    file.write_all(data).map_err(|e| format!("couldn't write to {}: {}", display, e.description()))?;

    Ok(())
} 

fn gen_random_private_key(name: &str) {
    let ctx = signing::create_context().expect("init context failure");

    let priv_key = ctx.new_random_private_key().expect("generate private key failure");
    write_file(&format!("{}.priv", name), priv_key.as_slice()).expect("write private key failure");
    
    let pub_key = ctx.get_public_key(&(*priv_key)).expect("get public key failure");
    write_file(&format!("{}.pub", name), pub_key.as_slice()).expect("write public key failure");

    println!("private key: {}", util::bytes_to_hex_str(priv_key.as_slice()));
    println!("public key: {}", util::bytes_to_hex_str(pub_key.as_slice()));
    println!("wallet: {}", wallet::new(pub_key.as_slice()));
}

fn load_private_key(file_name: &str) {
    let path = Path::new(file_name);
    let display = path.display();
    let bytes = fs::read(path).expect(&format!("read {} failure", display));
    let signer = signing::Signer::from_hex(&util::bytes_to_hex_str(&bytes)).expect("generate signer failure");

    let pub_key = signer.get_public_key().expect("get public key failure");

    println!("private key: {}", util::bytes_to_hex_str(&bytes));
    println!("public key: {}", pub_key);
    println!("wallet: {}", wallet::new(&util::hex_str_to_bytes(&pub_key).expect("convert hex to bytes failure")));

}

fn main() -> Result<(), String> {
    let matches = App::new("private key tool")
        .version(crate_version!())
        .author("Kigi Chang <kigi.chang@gmail.com>")
        .subcommand(
            SubCommand::with_name("gen")
                .about("generate a random private key")
                .arg(
                    Arg::with_name("name")
                        .long("name")
                        .short("n")
                        .required(true)
                        .help("name for private key")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("load")
                .about("load a private key")
                .arg(
                    Arg::with_name("input")
                        .long("input")
                        .short("i")
                        .required(true)
                        .help("private key file to load")
                        .takes_value(true),
                ),

        )
        .get_matches();

    if let Some(m) = matches.subcommand_matches("gen") {
        let prefix_name = m.value_of("name").unwrap();
        gen_random_private_key(prefix_name);
        println!("complete!");

    } else if let Some(m) = matches.subcommand_matches("load"){
        let file_name = m.value_of("input").unwrap();
        load_private_key(&file_name);
    }

    Ok(())
}
