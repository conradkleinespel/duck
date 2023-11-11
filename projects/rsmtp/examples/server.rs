extern crate rsmtp;

use rsmtp::server::commands::helo::get as get_helo_command;
use rsmtp::server::commands::HeloHandler;
use rsmtp::server::commands::HeloSeen;
use rsmtp::server::Server;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Clone)]
struct Container {
    helo_seen: bool,
}

impl Container {
    fn new() -> Container {
        Container { helo_seen: false }
    }
}

impl HeloSeen for Container {
    fn helo_seen(&mut self) -> bool {
        self.helo_seen
    }

    fn set_helo_seen(&mut self, helo_seen: bool) {
        self.helo_seen = helo_seen;
    }
}

impl HeloHandler for Container {
    fn handle_domain(&mut self, domain: &str) -> Result<(), ()> {
        println!("Got a client from domain: {:?}", domain);
        Ok(())
    }
}

fn main() {
    let container = Container::new();
    let mut server = Server::new(container);

    // Just one command for the example, but you can add more.
    // Look in `rsmtp::server::commands` for more commands.
    server.add_command(get_helo_command());

    // Hypothetical extension support.
    server.add_extension("STARTTLS");
    server.add_extension("BDAT");

    if let Err(_) = server.listen(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2525) {
        println!("Error.");
    }
}
