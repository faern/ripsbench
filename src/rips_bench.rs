use Config;
use ipnetwork::Ipv4Network;
use pnet;
use pnet::packet::ethernet::{EtherTypes, EtherType};
use progress;
use rips::{self, EthernetChannel, NetworkStack, Payload, TxError};
use rips::ethernet::{EthernetTx, EthernetPayload};
use rips::udp::UdpSocket;
use std::io::Write;
use std::process;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

lazy_static! {
    static ref DEFAULT_ROUTE: Ipv4Network = Ipv4Network::from_str("0.0.0.0/0").unwrap();
}

struct NullPayload;

impl EthernetPayload for NullPayload {
    fn ether_type(&self) -> EtherType {
        EtherTypes::Arp
    }
}

impl Payload for NullPayload {
    fn len(&self) -> usize {
        0
    }

    fn build(&mut self, buffer: &mut [u8]) {}
}

pub fn bench_ethernet(channel: EthernetChannel, config: &Config) {
    let mut printer = progress::Printer::new();
    let mut stack = create_stack(channel, config);
    let mut tx = stack.interface(&config.iface).unwrap().ethernet_tx(config.dst_mac);

    printer.print_title("Rips Ethernet sending");
    for packets_per_call in vec![1, 10, 100, 1000] {
        for bytes_per_packet in vec![28, config.mtu] {
            printer.print_line_description(&format!("Sending {}x{} bytes",
                                                    packets_per_call,
                                                    bytes_per_packet));
            let mut pkgs = 0;
            let mut bytes = 0;
            let mut next_print_second = 1;
            let timer = Instant::now();
            loop {
                let send_result = tx.send(packets_per_call, bytes_per_packet, NullPayload);
                match send_result {
                    Err(TxError::InvalidTx) => {
                        tx = stack.interface(&config.iface).unwrap().ethernet_tx(config.dst_mac)
                    }
                    Err(e) => panic!("Unable to send: {:?}", e),
                    _ => (),
                }
                pkgs += packets_per_call;
                bytes += packets_per_call * bytes_per_packet;

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
}

pub fn bench_udp(channel: EthernetChannel, config: &Config) {
    let mut printer = progress::Printer::new();
    let mut stack = create_stack(channel, config);

    stack.add_ipv4(&config.iface, config.src_net).unwrap();
    {
        let routing_table = stack.routing_table();
        routing_table.add_route(*DEFAULT_ROUTE, Some(config.gw), config.iface.clone());
    }

    let stack = Arc::new(Mutex::new(stack));
    let mut socket = UdpSocket::bind(stack, config.src).unwrap();

    printer.print_title("Rips UDP sending");

    for bytes_per_packet in vec![1, max_payload_in_one_frame(config), 65000] {
        printer.print_line_description(&format!("Sending {} bytes per packet", bytes_per_packet));
        let buffer = vec![0; bytes_per_packet];

        let mut pkgs = 0;
        let mut bytes = 0;
        let timer = Instant::now();
        let mut next_print = 1;
        loop {
            match socket.send_to(&buffer, config.dst) {
                Err(e) => {
                    eprintln!("Error while sending to the network: {}", e);
                    process::exit(1);
                }
                Ok(size) => {
                    pkgs += 1;
                    bytes += bytes_per_packet;
                }
            }
            let elapsed = timer.elapsed();
            if elapsed.as_secs() >= next_print {
                next_print += 1;
                printer.print_statistics(pkgs, bytes, elapsed);
            }
            if elapsed > config.duration {
                break;
            }
        }
        printer.end_line();
    }
}

fn create_stack(channel: EthernetChannel, config: &Config) -> NetworkStack {
    let mut stack = rips::NetworkStack::new();
    stack.add_interface(config.iface.clone(), channel).unwrap();
    stack
}

fn max_payload_in_one_frame(config: &Config) -> usize {
    config.mtu - pnet::packet::udp::UdpPacket::minimum_packet_size() -
    pnet::packet::ipv4::Ipv4Packet::minimum_packet_size() -
    pnet::packet::ethernet::EthernetPacket::minimum_packet_size()
}
