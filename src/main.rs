use std::convert::TryFrom;
use std::io::{self, BufRead};

use clap::error;

mod core;

fn main() {
    use crate::core::domain::network::entities::ProtocolMessage;

    println!("FerrisShare REPL: tape une commande protocole (ligne vide pour quitter).");
    println!(
        "Exemples: HELLO foo.txt 42 | OK | NOPE raison | YEET 0 4096 12345 | OK-HOUSTEN 0 | MISSION-ACCOMPLISHED | SUCCESS | ERROR oops | BYE-RIS"
    );

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("I/O error: {e}");
                break;
            }
        };
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            println!("Bye.");
            break;
        }

        match ProtocolMessage::try_from(trimmed.as_str()) {
            Ok(msg) => {
                println!("Parsed => {:?}", msg);
                let response: String = String::from(msg);
                println!("Response => {}", response);
            }
            Err(err) => {
                println!("Parse error => {:?}", err);
                let error_response = String::from(err);
                println!("Error response => {}", error_response);
            }
        }
    }
}
