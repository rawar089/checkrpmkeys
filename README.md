checkrpmkeys
============

Tiny tui/cli tool to check installed RPM repository GPG signing keys for expirations. The tool
queries the local RPM database for 'gpg-pubkey' packages with the rpm command. 
It decodes the internal cryptographic data using the sequoia crate and analyzes expiration states.

Note the sequoia-openpgp crate is using the crypto backend nettle by default. In version 4 nettle has removed pgp support.
From version 0.5.0 on the pure Rust `crypto-rust` backend will be used instead. The use of the `crypto-rust` crate in general 
is not recommended, as it has not yet reached the level of maturity of other external backends such as `Nettle` or `OpenSSL`.
However, its use is justifiable for this application, which deliberately avoids network functions and merely displays information.

If started with an option the tool is working in cli mode and exits after the information has been written to stdout in the specified format.

Otherwise the default terminal ui mode is active and shows the installed keys. The tui has sortable columns, a search filter,
 a detail and help popup. It is using the ratatui/crossterm crates.

### Usage:

```
Usage: checkrpmkeys [OPTIONS]

Options:
  -j, --json      Output the raw records array as pretty-printed JSON payload
  -g, --generate  Generate a bash shell script containing 'rpm -e' removal targets for expired keys
  -y, --yaml      Output the raw records array as pretty-printed JSON payload
  -h, --help      Print help (see more with '--help')
  -V, --version   Print version

```

### Language / i18n

- On startup the language is auto-detected from `LC_ALL` → `LC_MESSAGES` →
  `LANG` → `LANGUAGE`, falling back to English. 
   To force the language to German try: `LANG=de_DE.UTF-8 cargo run`.
- Press `l` at any time to cycle the language manually; the current
  language shows as a badge (`[DE]`) in the title bar.
- The current version supports English, German and French (translated with AI).

## Build & run

```
cargo run
```

### Requirements:

- Linux OS with installed rpm command

