# IEEE 802.15.4 Sniffer

This is an extcap to be used with the `sniffer` example. Make sure to configure the channel in the `sniffer` example.

To use it build via `cargo build --release` and copy the resulting executable to the Wireshark's `extcap` folder.

Then you should see two new capture interface in Wireshark

If you are running the example this capture interface can connect via serialport to give you insights on what is going on.

By default it tries to identify exactly one serialport. If that doesn't work for you, you can configure the serialport via the Wireshark UI.

In Wireshark use `ITU-T-CRC-16` as `FCS format`
