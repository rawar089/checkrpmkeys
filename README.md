checkrpmkeys
============

Tiny tool to check installed RPM repository GPG signing keys for expirations. The tool
queries the local RPM database for 'gpg-pubkey' packages with the rpm command, 
decodes the packages internal cryptographic data using the sequoia crate, and analyzes expiration states.
Note the sequoia-openpgp crate is used with the default crypto backend nettle.  
### Usage:

```
Usage: checkrpmkeys [OPTIONS]

Options:
  -j, --json      Output the completely raw records array as pretty-printed JSON payload for debugging
  -g, --generate  Generate an actionable bash shell script containing exact 'rpm -e' removal targets for expired keys
  -h, --help      Print help (see more with '--help')
  -V, --version   Print version

```

### Requirements:

- Linux OS with installed rpm command
- libhogweed6 
- libnettle8

