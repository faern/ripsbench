use Config;
use progress;

use rips::EthernetChannel;

use std::time::Instant;

pub fn bench(channel: EthernetChannel, config: &Config) {
    let EthernetChannel(mut sender, _receiver) = channel;
    let mut printer = progress::Printer::new();

    printer.print_title("Raw pnet datalink sending");
    for packets_per_call in vec![1, 10, 100, 1000] {
        for bytes_per_packet in vec![42, 1514] {
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
