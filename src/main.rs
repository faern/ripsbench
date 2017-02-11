#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate pnet;
extern crate ipnetwork;
extern crate rips;

#[macro_use]
mod args;
mod progress;

mod pnet_bench;
mod rips_bench;

use ipnetwork::Ipv4Network;
use rips::Interface;
use rips::ethernet::MacAddr;
use std::net::{SocketAddrV4, Ipv4Addr};
use std::time::Duration;

lazy_static! {
    static ref DEFAULT_IFACE: Interface = Interface { name: "test0".to_owned(), mac: *SRC_MAC };
    static ref SRC_MAC: MacAddr = MacAddr::new(11, 22, 33, 44, 55, 66);
    static ref SRC_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 2);
    static ref SRC_NET: Ipv4Network = Ipv4Network::new(*SRC_IP, 24).unwrap();
    static ref SRC_SOCKETADDR: SocketAddrV4 = SocketAddrV4::new(*SRC_IP, 0);

    static ref GW: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);

    static ref DST_MAC: MacAddr = MacAddr::new(99, 88, 77, 66, 55, 44);
    static ref DST_LAN_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 15);
    static ref DST_LAN_SOCKETADDR: SocketAddrV4 = SocketAddrV4::new(*DST_LAN_IP, 8080);
}

#[derive(Debug)]
pub struct Config {
    pub duration: Duration,
    pub iface: rips::Interface,
    pub mtu: usize,
    pub src_mac: MacAddr,
    pub src: SocketAddrV4,
    pub src_net: Ipv4Network,
    pub dst_mac: MacAddr,
    pub dst: SocketAddrV4,
    pub gw: Ipv4Addr,
    pub size_min: bool,
    pub size_mtu: bool,
    pub size_max: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            duration: Duration::new(10, 0),
            iface: (*DEFAULT_IFACE).clone(),
            mtu: 1500,
            src_mac: *SRC_MAC,
            src: *SRC_SOCKETADDR,
            src_net: *SRC_NET,
            dst_mac: *DST_MAC,
            dst: *DST_LAN_SOCKETADDR,
            gw: *GW,
            size_min: false,
            size_mtu: false,
            size_max: false,
        }
    }
}

fn main() {
    let args = args::ArgumentParser::new();

    let (_, iface) = args.get_iface();
    let src_net = args.get_src_net();
    let src_port = args.get_src_port();
    let src = SocketAddrV4::new(src_net.ip(), src_port);

    let mut config = Config::new();
    config.iface = iface;
    config.mtu = args.get_mtu();
    config.src = src;
    config.src_net = src_net;
    config.dst = args.get_dst();
    config.gw = args.get_gw();
    for size in args.get_sizes() {
        match size {
            args::Size::Min => config.size_min = true,
            args::Size::Mtu => config.size_mtu = true,
            args::Size::Max => config.size_max = true,
        }
    }

    println!("CONFIG: {:?}", &config);

    for protocol in args.get_protocols() {
        match protocol {
            args::Protocol::Pnet => pnet_bench::bench(args.create_channel(), &config),
            args::Protocol::Ethernet => rips_bench::bench_ethernet(args.create_channel(), &config),
            args::Protocol::Ipv4 => rips_bench::bench_ipv4(args.create_channel(), &config),
            args::Protocol::Udp => rips_bench::bench_udp(args.create_channel(), &config),
        }
    }
}
