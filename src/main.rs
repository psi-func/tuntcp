use etherparse;
use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;
use tun_tap::{Iface, Mode};

mod tcp;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let nic = Iface::new("tun0", Mode::Tun)?;
    let mut connections: HashMap<Quad, tcp::State> = Default::default();
    let mut buf = [0_u8; 1504];

    loop {
        let nbytes = nic.recv(buf.as_mut_slice())?;

        let _eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);

        if eth_proto != 0x0800 {
            // no ipv4
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                if iph.protocol() != 0x06 {
                    // no tcp
                    continue;
                }

                let ip_hdr_sz = iph.slice().len();
                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + ip_hdr_sz..nbytes]) {
                    Ok(tcph) => {
                        let datai = 4 + ip_hdr_sz + tcph.slice().len();
                        connections
                            .entry(Quad {
                                src: (src, tcph.source_port()),
                                dst: (dst, tcph.destination_port()),
                            })
                            .or_default()
                            .on_packet(iph, tcph, &buf[datai..nbytes]);
                    }
                    Err(e) => {
                        eprintln!("Some weird tcp packet: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Some weird ip packet: {e}");
            }
        }
    }
}
