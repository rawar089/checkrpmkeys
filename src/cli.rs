use chrono::Local;
use crate::data::GpgKeyRecord;
use crate::Args;
use anyhow::Result;

fn print_json(keys: &[GpgKeyRecord]) {
    match serde_json::to_string_pretty(keys) {
        Ok(json_str) => println!("{}", json_str),
        Err(e) => eprintln!("Failed to serialize GPG key records to JSON: {}", e),
    }
}

fn print_yaml(keys: &[GpgKeyRecord]) {
    match serde_yaml::to_string(keys) {
        Ok(yaml_str) => println!("{}", yaml_str),
        Err(e) => eprintln!("Failed to serialize GPG key records to YAML: {}", e),
    }
}

fn generate_removal_script(keys: &[GpgKeyRecord]) {
    println!("#!/usr/bin/env bash");
    println!("# Generated shell script to remove expired GPG pubkey RPM packages");
    println!(
        "# Generated on: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    if keys.is_empty() {
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

    for key in keys {
        if key.expired {
            println!("# Owner: {}", key.uid);
            println!("# Expired: {}", key.expires);
            println!("echo 'Erasing package {}...'", key.package_name);
            println!("rpm -e '{}'\n", key.package_name);
        }
    }

    println!("echo 'Cleanup complete!'");
}

pub fn run_cli(args: &Args,gpg_keys: Vec<GpgKeyRecord>) -> Result<()>{
    if args.json {
        print_json(&gpg_keys);
    } else if args.yaml  {
        print_yaml(&gpg_keys)
    } else if args.generate {
        generate_removal_script(&gpg_keys);
    }
    Ok(())
}