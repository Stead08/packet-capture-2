use std::env;

use log::error;
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;

use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::Packet;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("Please specify target interface bane");
        std::process::exit(1);
    }
    let interface_name = &args[1];

    // select interface
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == *interface_name)
        .expect("Failed to get interface");

    // get datalink channel
    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Failed to create datalink channel {}", e)
    };

    loop {
        match rx.next() {
            Ok(frame) => {
                let frame = EthernetPacket::new(frame).unwrap();
                let destination = &frame.get_destination();
                let source = &frame.get_source();
                let ether_type = &frame.get_ethertype();
                let ether_type_hex = &frame.get_ethertype().0;

                let raw_packet = &frame.packet();
                let len = raw_packet.len();
                let mut packet = vec![0u8; len * 2];
                if let Err(e) = hex::encode_to_slice(raw_packet, &mut packet) {
                    error!("Failed to encode to slice: {}", e);
                    continue;
                }
                let packet_chunks: Vec<_> = packet.chunks(2).collect();
                let packet: Vec<String> = packet_chunks.into_iter().map(|c| String::from_utf8(c.to_vec()).unwrap()).collect();
                let packet = packet.join(" ");

                println!(
                    "Destination: {}, Source: {}, EtherType: {}({:#x})",
                    destination,
                    source,
                    ether_type,
                    ether_type_hex,
                );
                println!();
            },
            Err(e) => error!("Failed to read {}", e)
        }
    }
}
