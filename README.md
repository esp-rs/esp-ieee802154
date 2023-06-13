# esp-ieee802154

Low-level IEEE802.15.4 driver for the ESP32-C6 and ESP32-H2.

## Running examples

`cargo run --release --example EXAMPLENAME --features esp32c6`

Available examples

- receive_all_frames: print all received frames on channel 15 in promiscuous mode
- receive_frame: print all received frames on channel 11, pan 0x4242 sent to short address 0x2323
- send_broadcast_frame: send broadcast frames on channel 11
- send_frame: send regular frames on channel 11, pan 0x4242 to short address 0x2323

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
