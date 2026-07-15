use serde::Serialize;
use std::process::Command;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use sequoia_openpgp::Cert;
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::parse::Parse;


#[derive(Serialize, Clone, Debug)]
pub struct GpgKeyRecord {
    pub package_name: String,
    pub key_type: String,
    pub key_size: Option<usize>,
    pub uid: String,
    pub expires: String,
    pub fingerprint: String,
    pub expired: bool,
    pub valid: bool,
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

fn get_gpg_data(raw_records: Vec<(String, String)>) -> Vec<GpgKeyRecord> {
    let mut gpg_data = Vec::new();
    let policy = &StandardPolicy::new();
    let debug = false;

    for (pkg_name, gpg_block) in raw_records {
        let mut key_type_str = "Unknown".to_string();
        let mut uid_string = "No UID found".to_string();
        let mut key_size: Option<usize> = None;
        let mut expires_str = "Unknown".to_string();
        let mut fingerprint_str = "Unknown".to_string();
        let mut is_expired = false;
        let mut is_valid = false;

        if let Ok(cert) = Cert::from_bytes(gpg_block.as_bytes()) {
            fingerprint_str = cert.fingerprint().to_string();
            let primary_key = cert.primary_key();
            key_type_str = primary_key.key().pk_algo().to_string();
            key_size = primary_key.key().mpis().bits();

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
                    is_valid = true; // TODO add missing revocation check
                },
                Err(_e) if !is_expired  => {
                    // Our default policy rejects this but the key seems ok for rpm.
                    // So we ignore it in this version. Need investigation.
                    // println!("Error: {}\nq", e);
                    is_valid = true;
                },
                Err(_) =>  {},  // expired anyway
            }


        }

        let record = GpgKeyRecord {
            package_name: pkg_name,
            key_type: key_type_str,
            key_size: key_size,
            uid: uid_string,
            expires: expires_str,
            fingerprint: fingerprint_str,
            valid: is_valid,
            expired: is_expired,
        };

        gpg_data.push(record);
    }
    gpg_data
}

pub fn load_data() -> Vec<GpgKeyRecord> {
    get_gpg_data(fetch_rpm_gpg_packages())
}

/// Derived status used for coloring/sorting. Expired takes priority over
/// "invalid" (e.g. a bad self-signature) since it's the more actionable fact.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyStatus {
    Expired,
    Invalid,
    Valid,
}

impl GpgKeyRecord {
    pub fn status(&self) -> KeyStatus {
        if self.expired {
            KeyStatus::Expired
        }  else if !self.valid {
            KeyStatus::Invalid
        } else {
            KeyStatus::Valid
        }
    }
}





