# ASN.1 to ROS.msg
Backend for the rasn compiler, which converts ASN.1 files to ROS message files. Support mainly for ETSI ITS messages.

## Build
Get the latest version of the [rasn compiler](https://github.com/librasn/compiler). It should be in the same parent directory as this repository.

## Usage
Simply run `cargo run [ASN.1 file ...]`. The corresponding ROS `.msg`s will be generated in the `out` directory.
