# ASN.1 to ROS Compiler
Backends for the rasn compiler, which converts ASN.1 files to ROS message files, and generates support conversion headers for translation between asn1c structs and ROS structs. 

Support mainly for ETSI ITS messages.

## Build
Get the latest version of the [rasn compiler](https://github.com/librasn/compiler). It should be in the same parent directory as this repository.

## Usage
To generate the ROS `.msg`s run `cargo run --bin asn1-to-ros-msgs -p <PDU> [ASN.1 files ...]`, where `<PDU>` is the main PDU name used as a reference (e.g. `cam`, `denm`).

To generate the conversion headers run `cargo run --bin asn1-to-ros-conversion-headers -p <PDU> [ASN.1 files ...]`.

The corresponding ROS `.msg`s and conversion headers will be generated in the `out` directory.
