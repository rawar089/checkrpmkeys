use chrono::{DateTime, Local};
use clap::Parser;
use owo_colors::OwoColorize;
use sequoia_openpgp::Cert;
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::parse::Parse;
use serde::Serialize;
use std::process::Command;
use std::time::SystemTime;
use tabled::{
    Table, Tabled,
    settings::{Color, Modify, Style, Width, object::{Columns, Rows},
    },
};

#[derive(Tabled, Serialize, Clone)]
#[tabled(rename_all = "PascalCase")]
struct GpgKeyRecord {
    #[tabled(rename = "Package Name")]
    package_name: String,
    #[tabled(rename = "Key Type")]
    key_type: String,
    #[tabled(rename = "Primary UID (Owner)")]
    uid: String,
    #[tabled(rename = "Expiration Date")]
    expires: String,
    #[tabled(rename = "Fingerprint")]
    fingerprint: String,
    expired: bool,
    valid: bool,
}

// Define the command-line argument schema using Clap
#[derive(Parser, Debug)]
#[command(
    name = "checkrpmkeys",
    author = "rawar089",
    version = "0.1",
    about = "Checks installed RPM repository GPG signing keys for expirations.",
    long_about = "Queries the local RPM database for 'gpg-pubkey' packages, decodes their internal cryptographic profiles using sequoia, and analyzes expiration states."
)]
struct Args {
    /// Output the completely raw records array as pretty-printed JSON payload for debugging
    #[arg(long, conflicts_with = "generate")]
    json: bool,

    /// Generate an actionable bash shell script containing exact 'rpm -e' removal targets for expired keys
    #[arg(long, conflicts_with = "json")]
    generate: bool,
}

fn fetch_rpm_gpg_packages() -> Vec<(String, String)> {
    let output = execute_rpm();

    let mut records = Vec::new();
    if let Ok(out) = output {
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        let raw_blocks = stdout_str.split("\n---END_RPM_RECORD---\n");

        for block in raw_blocks {
            let mut lines = block.trim().lines();
            if let Some(pkg_name) = lines.next() {
                if pkg_name.is_empty() {
                    continue;
                }
                let description: Vec<&str> = lines.collect();
                records.push((pkg_name.to_string(), description.join("\n")));
            }
        }
    } else {
        output.expect("Failed to execute rpm command");
    }
    records
}

fn execute_rpm() -> Result<std::process::Output, std::io::Error> {
        Command::new("rpm")
        .args(["-qa",
            "gpg-pubkey*",
            "--queryformat",
            "%{NAME}-%{VERSION}-%{RELEASE}\n%{DESCRIPTION}\n---END_RPM_RECORD---\n",
        ]).output()
}

fn get_expiration_timestamp(cert: &Cert) -> Option<SystemTime> {
    let primary_key = cert.primary_key().key();

    for userid in cert.userids() {
        for sig in userid.self_signatures() {
            if let Some(duration) = sig.key_validity_period() {
                return Some(primary_key.creation_time() + duration);
            }
        }
    }
    None
}
fn get_gpg_data(raw_records: Vec<(String, String)>) -> Vec<(GpgKeyRecord, bool)> {
    let mut gpg_data = Vec::new();
    let policy = &StandardPolicy::new();
    let debug = false;

    for (pkg_name, gpg_block) in raw_records {
        let mut key_type_str = "Unknown".to_string();
        let mut uid_string = "No UID found".to_string();
        let mut expires_str = "Unknown".to_string();
        let mut fingerprint_str = "Unknown".to_string();
        let mut is_expired = false;
        let mut is_valid = false;

        if let Ok(cert) = Cert::from_bytes(gpg_block.as_bytes()) {
            fingerprint_str = cert.fingerprint().to_string();
            let primary_key = cert.primary_key();
            key_type_str = primary_key.key().pk_algo().to_string();
            let uids: Vec<String> = cert.userids().map(|uid| uid.userid().to_string()).collect();
            if !uids.is_empty() {
                uid_string = uids.join(", ");
            }

            let key_expiry = get_expiration_timestamp(&cert);
            match key_expiry {
                Some(expiration) => {
                    let expiry: DateTime<Local> = expiration.into();
                    let now: DateTime<Local> = Local::now();
                    expires_str = expiry.format("%Y-%m-%d %H:%M:%S").to_string();

                    if expiry < now {
                        is_expired = true;
                    }
                    if debug {
                        println!("Key {} expires at: {}", uid_string, expires_str);
                    }
                }
                None => {
                    expires_str = "Never Expires".to_string();
                    if debug {
                        println!("Key {} {}: ", uid_string, expires_str);
                    }
                }
            }

            match cert.with_policy(policy,None) {
                Ok(_valid_cert) => {  // cert is valid with policy
                                      is_valid = true; // TODO missing revocation check
                                   },
                Err(e) if !is_expired => { // Strange error
                                                  println!("Error: {}\nq", e);
                                               },
                Err(_) =>  {},  // expired anyway
            }


        }

        let record = GpgKeyRecord {
            package_name: pkg_name,
            key_type: key_type_str,
            uid: uid_string,
            expires: expires_str,
            fingerprint: fingerprint_str,
            valid: is_valid,
            expired: is_expired,
        };

        gpg_data.push((record, is_expired));
    }
    gpg_data
}

fn print_records(records: &[GpgKeyRecord]) {
    match serde_json::to_string_pretty(records) {
        Ok(json_str) => println!("{}", json_str),
        Err(e) => eprintln!("Failed to serialize GPG key records to JSON: {}", e),
    }
}

fn generate_removal_script(expired_keys: &[GpgKeyRecord]) {
    println!("#!/usr/bin/env bash");
    println!("# Generated shell script to remove expired GPG pubkey RPM packages");
    println!(
        "# Generated on: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    if expired_keys.is_empty() {
        println!(
            "echo 'No expired repository GPG keys were detected on this system. Nothing to do.'"
        );
        return;
    }

    println!("if [ \"$EUID\" -ne 0 ]; then");
    println!("  echo 'Error: This script must be run as root/sudo to erase RPM packages.' >&2");
    println!("  exit 1");
    println!("fi\n");

    println!("echo 'Starting removal of expired repository GPG keys...'\n");

    for key in expired_keys {
        println!("# Owner: {}", key.uid);
        println!("# Expired: {}", key.expires);
        println!("echo 'Erasing package {}...'", key.package_name);
        println!("rpm -e '{}'\n", key.package_name);
    }

    println!("echo 'Cleanup complete!'");
}

fn main() {
    let args = Args::parse();

    let optional_output = args.json || args.generate;

    if !optional_output {
        println!("{}", "Fetching installed GPG keys from RPM database...".cyan()
        );
    }

    let raw_data = fetch_rpm_gpg_packages();
    if raw_data.is_empty() {
        if !optional_output {
            println!("{}", "No gpg-pubkey packages found or rpm command missing.".yellow()
            );
        } else if args.json {
            println!("[]");
        }
    }

    let enriched_data = get_gpg_data(raw_data);

    let mut table_rows = Vec::new();
    let mut expired_keys = Vec::new();
    let mut expired_indices = Vec::new();
    let mut expired_count = 0;

    for (idx, (record, is_expired)) in enriched_data.into_iter().enumerate() {
        if is_expired {
            expired_count += 1;
            expired_indices.push(idx + 1);
            expired_keys.push(record.clone());
        }
        table_rows.push(record);
    }

    if args.json {
        print_records(&table_rows);
    } else if args.generate {
        generate_removal_script(&expired_keys);
    } else {
        let mut table = Table::new(table_rows);
        table.with(Style::modern());
        //table.with(Modify::new(Columns::single(4)).with(Width::truncate(45)));
        table.with(Modify::new(Columns::single(0)).with(Width::wrap(10).keep_words(true)));
        table.with(Modify::new(Columns::single(1)).with(Width::wrap(5).keep_words(true)));
        table.with(Modify::new(Columns::single(2)).with(Width::wrap(30).keep_words(true)));
        table.with(Modify::new(Columns::single(3)).with(Width::wrap(10).keep_words(true)));
        table.with(Modify::new(Columns::single(4)).with(Width::wrap(15).keep_words(true)));
        for row_idx in expired_indices {
            table.modify(Rows::single(row_idx), Color::BG_RED);
        }

        println!("\n{}", table);

        if expired_count > 0 {
            println!("\n{}", format!(
                               "Alert: Found {} expired key(s). Updates may be required.",
                               expired_count).bold().red());
        } else {
            println!("\n{}", "Success: All analyzed public keys are secure and active."
                    .bold().green());
        }
    }
}
