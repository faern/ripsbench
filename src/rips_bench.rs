use Config;
use ipnetwork::Ipv4Network;
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ip::IpNextHeaderProtocols;
use progress;
use rips::{self, EthernetChannel, NetworkStack, TxError};
use rips::{CustomPayload, Tx};
use rips::ethernet::{EthernetTx, EthernetFields};
// use rips::ipv4::{Ipv4Tx, BasicIpv4Payload};
// use rips::udp::UdpSocket;
use std::process;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

lazy_static! {
    static ref DEFAULT_ROUTE: Ipv4Network = Ipv4Network::from_str("0.0.0.0/0").unwrap();
}

pub fn bench_ethernet(channel: EthernetChannel, config: &Config) {
    let mut printer = progress::Printer::new();
    let mut stack = create_stack(channel, config);
    let mut interface = stack.interface(&config.iface).unwrap();
    let mut tx = interface.ethernet_tx(config.dst_mac);

    printer.print_title("Rips Ethernet sending");
    let buffer = vec![0; 1000 * 1500];
    let mut invalid_tx_count = 0;
    for packets_per_call in vec![1, 10, 100, 1000] {
        for bytes_per_packet in packet_sizes(config, Protocol::Ethernet) {
            printer.print_line_description(&format!("Sending {}x{} bytes",
                                                    packets_per_call,
                                                    bytes_per_packet));
            let mut pkgs = 0;
            let mut bytes = 0;
            let mut next_print_second = 1;
            let timer = Instant::now();
            loop {
                let total_bytes = packets_per_call * bytes_per_packet;
                let mut payload = CustomPayload::with_packet_size(EthernetFields(EtherTypes::Ipv4),
                                                                  bytes_per_packet,
                                                                  &buffer[..total_bytes]);
                match tx.send(&mut payload) {
                    None => {
                        invalid_tx_count += 1;
                        tx = interface.ethernet_tx(config.dst_mac);
                    }
                    Some(Err(e)) => panic!("Unable to send: {:?}", e),
                    _ => {
                        pkgs += packets_per_call;
                        bytes += total_bytes;
                    }
                }

                let elapsed = timer.elapsed();
                if elapsed.as_secs() >= next_print_second {
                    printer.print_statistics(pkgs, bytes, elapsed);
                    next_print_second += 1;
                }
                if elapsed > config.duration {
                    break;
                }
            }
            printer.end_line();
        }
    }
    println!("Benchmark resulted in {} InvalidTx", invalid_tx_count);
}

// pub fn bench_ipv4(channel: EthernetChannel, config: &Config) {
//     let mut printer = progress::Printer::new();
//     let mut stack = create_stack(channel, config);
//     let mut tx = stack.ipv4_tx(*config.dst.ip()).unwrap();

//     printer.print_title("Rips IPv4 sending");

//     for bytes_per_packet in packet_sizes(config, Protocol::Ipv4) {
// printer.print_line_description(&format!("Sending {} bytes per
// packet", bytes_per_packet));
//         let buffer = vec![0; bytes_per_packet];

//         let mut pkgs = 0;
//         let mut bytes = 0;
//         let timer = Instant::now();
//         let mut next_print = 1;
//         loop {
// let payload = BasicIpv4Payload::new(IpNextHeaderProtocols::Igmp,
// &buffer[..]);
//             match tx.send(payload) {
//                 Err(TxError::InvalidTx) => {
//                     tx = stack.ipv4_tx(*config.dst.ip()).unwrap();
//                 }
//                 Err(e) => {
//                     eprintln!("Error while sending to the network: {}", e);
//                     process::exit(1);
//                 }
//                 Ok(_size) => {
//                     pkgs += 1;
//                     bytes += bytes_per_packet;
//                 }
//             }
//             let elapsed = timer.elapsed();
//             if elapsed.as_secs() >= next_print {
//                 next_print += 1;
//                 printer.print_statistics(pkgs, bytes, elapsed);
//             }
//             if elapsed > config.duration {
//                 break;
//             }
//         }
//         printer.end_line();
//     }
// }

// pub fn bench_udp(channel: EthernetChannel, config: &Config) {
//     let mut printer = progress::Printer::new();
//     let stack = create_stack(channel, config);

//     let stack = Arc::new(Mutex::new(stack));
//     let mut socket = UdpSocket::bind(stack, config.src).unwrap();

//     printer.print_title("Rips UDP sending");

//     for bytes_per_packet in packet_sizes(config, Protocol::Udp) {
// printer.print_line_description(&format!("Sending {} bytes per
// packet", bytes_per_packet));
//         let buffer = vec![0; bytes_per_packet];

//         let mut pkgs = 0;
//         let mut bytes = 0;
//         let timer = Instant::now();
//         let mut next_print = 1;
//         loop {
//             match socket.send_to(&buffer, config.dst) {
//                 Err(e) => {
//                     eprintln!("Error while sending to the network: {}", e);
//                     process::exit(1);
//                 }
//                 Ok(_size) => {
//                     pkgs += 1;
//                     bytes += bytes_per_packet;
//                 }
//             }
//             let elapsed = timer.elapsed();
//             if elapsed.as_secs() >= next_print {
//                 next_print += 1;
//                 printer.print_statistics(pkgs, bytes, elapsed);
//             }
//             if elapsed > config.duration {
//                 break;
//             }
//         }
//         printer.end_line();
//     }
// }

fn create_stack(channel: EthernetChannel, config: &Config) -> NetworkStack {
    let mut stack = rips::NetworkStack::new();
    stack.add_interface(config.iface.clone(), channel).unwrap();
    stack.add_ipv4(&config.iface, config.src_net).unwrap();
    {
        let mut routing_table = stack.routing_table();
        routing_table.add_route(*DEFAULT_ROUTE, Some(config.gw), config.iface.clone());
    }
    stack
}

#[derive(PartialEq, Eq, Debug)]
enum Protocol {
    Ethernet,
    Ipv4,
    Udp,
}

fn packet_sizes(config: &Config, protocol: Protocol) -> Vec<usize> {
    let mut sizes = Vec::new();
    if config.size_min {
        sizes.push(match protocol {
            Protocol::Ethernet => 20 + 8,
            Protocol::Ipv4 => 8,
            Protocol::Udp => 0,
        });
    }
    if config.size_mtu {
        let size = config.mtu -
                   match protocol {
            Protocol::Ethernet => 0,
            Protocol::Ipv4 => 20,
            Protocol::Udp => 20 + 8,
        };
        sizes.push(size);
    }
    if config.size_max && protocol != Protocol::Ethernet {
        sizes.push(65000);
    }
    sizes
}
