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
use rips::{MacAddr, Interface};
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr, IpAddr};
use std::str::FromStr;
use std::time::Duration;

lazy_static! {
    static ref DEFAULT_IFACE: Interface = Interface { name: "test0".to_owned(), mac: *SRC_MAC };
    static ref SRC_MAC: MacAddr = MacAddr::new(11, 22, 33, 44, 55, 66);
    static ref SRC_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 2);
    static ref SRC_NET: Ipv4Network = Ipv4Network::new(*SRC_IP, 24).unwrap();
    static ref SRC_SOCKETADDR: SocketAddr = SocketAddr::new(IpAddr::V4(*SRC_IP), 0);

    static ref GW: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);

    static ref DST_MAC: MacAddr = MacAddr::new(99, 88, 77, 66, 55, 44);
    static ref DST_LAN_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 15);
    static ref DST_WAN_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 1);
    static ref DST_LAN_SOCKETADDR: SocketAddr = SocketAddr::new(IpAddr::V4(*DST_LAN_IP), 8080);
    static ref DST_WAN_SOCKETADDR: SocketAddr = SocketAddr::new(IpAddr::V4(*DST_WAN_IP), 8080);
}

pub struct Config {
    pub duration: Duration,
    pub iface: rips::Interface,
    pub mtu: usize,
    pub src_mac: MacAddr,
    pub src: SocketAddr,
    pub src_net: Ipv4Network,
    pub dst_mac: MacAddr,
    pub dst: SocketAddr,
    pub gw: Ipv4Addr,
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
        }
    }
}

fn main() {
    let args = args::ArgumentParser::new();

    let (_, iface) = args.get_iface();
    let src_net = args.get_src_net();
    let src_port = args.get_src_port();
    let src = SocketAddr::V4(SocketAddrV4::new(src_net.ip(), src_port));

    let mut config = Config::new();
    config.iface = iface;
    config.mtu = args.get_mtu();
    config.src = src;
    config.src_net = src_net;
    config.dst = args.get_dst();
    config.gw = args.get_gw();

    // pnet_bench::bench(args.create_channel(), &config);
    rips_bench::bench_ethernet(args.create_channel(), &config);
    // rips_bench::bench_udp(args.create_channel(), &config);
}
