use Config;
use progress;

use rips::EthernetChannel;

use std::time::Instant;

pub fn bench(channel: EthernetChannel, config: &Config) {
    let mut sender = channel.sender;
    let mut printer = progress::Printer::new();

    printer.print_title("Raw pnet datalink sending");
    for packets_per_call in vec![1, 10, 100, 1000] {
        for bytes_per_packet in packet_sizes(config) {
            printer.print_line_description(&format!("Sending {}x{} bytes",
                                                    packets_per_call,
                                                    bytes_per_packet));
            let mut pkgs = 0;
            let mut bytes = 0;
            let mut next_print_second = 1;
            let timer = Instant::now();
            loop {
                sender.build_and_send(packets_per_call, bytes_per_packet, &mut |_packet| {})
                    .expect("Too small buffer")
                    .expect("Unable to send");
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

fn packet_sizes(config: &Config) -> Vec<usize> {
    let mut sizes = Vec::new();
    if config.size_min {
        sizes.push(42);
    }
    if config.size_mtu {
        sizes.push(config.mtu + 14);
    }
    sizes
}
