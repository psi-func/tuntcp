use std::io;

#[non_exhaustive]
pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}


impl Default for State {
    fn default() -> Self {
        // State closed
        Self::Listen
    }
}


/// RFC-793 S3.2 F4
/// ```
/// Send Sequence Space
///
/// 1         2          3          4
/// ----------|----------|----------|----------
///        SND.UNA    SND.NXT    SND.UNA
///                             +SND.WND
///
/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers of unacknowledged data
/// 3 - sequence numbers allowed for new data transmission
/// 4 - future sequence numbers which are not yet allowed
///     Send Sequence Space
///
///         Figure 4.
/// ```
#[derive(Default)]
struct SendSequenceSpace {
    /// send unacknowledged
    una: u32,
    /// send next
    nxt: u32,
    /// send window
    wnd: u16,
    /// send urgent pointer
    up: bool,
    /// segment sequence number used for last window update
    wl1: u32,
    /// segment acknowledgment number used for last window update
    wl2: u32,
    /// initial send sequence number
    iss: u32,
}

/// RFC-793 S3.2 F5
/// ```
/// Receive Sequence Space
///
/// 1          2          3
/// ----------|----------|----------
///        RCV.NXT    RCV.NXT
///                  +RCV.WND
///
/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers allowed for new reception
/// 3 - future sequence numbers which are not yet allowed
///
///      Receive Sequence Space
///
///            Figure 5.
/// ```
#[derive(Default)]
struct RecvSequenceSpace {
    /// receive next
    nxt: u32,
    /// receive window
    wnd: u16,
    /// receive urgent pointer
    up: bool,
    /// initial receive sequence number
    irs: u32,
}

pub struct Connection {
    state: State,
    recv: RecvSequenceSpace,
    send: SendSequenceSpace,
}

impl Connection {
    pub fn accept() -> io::Result<Self> {
        
    }


    pub fn on_packet<'a>(
        &mut self,
        nic: &tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<usize> {
        eprintln!(
            "{}:{} - {}:{} {}b",
            iph.source_addr(),
            tcph.source_port(),
            iph.destination_addr(),
            tcph.destination_port(),
            data.len(),
        );

        let buf = [0_u8; 1500];
        match self.state {
            State::Closed => Ok(0),
            State::Listen => {
                if !tcph.syn() {
                    // only syn expected
                    return Ok(0);
                }

                // keep track of sender info
                self.recv.irs = tcph.sequence_number();
                self.recv.nxt = tcph.sequence_number() + 1;
                self.recv.wnd = tcph.window_size();

                // decide sending them
                self.send.iss = 0;
                self.send.una = self.send.iss;
                self.send.nxt = self.send.iss + 1;
                self.send.wnd = 10;
                // start establishing a connection
                let mut syn_ack = etherparse::TcpHeader::new(
                    tcph.destination_port(),
                    tcph.source_port(),
                    self.send.iss,
                    self.send.wnd,
                );

                syn_ack.acknowledgment_number = self.recv.nxt;
                syn_ack.syn = true;
                syn_ack.ack = true;
                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    etherparse::ip_number::TCP,
                    iph.destination(),
                    iph.source(),
                );
                let unwritten = {
                    let mut unwritten = &mut buf[..];
                    ip.write(&mut unwritten);
                    syn_ack.write(&mut unwritten);
                    unwritten.len()
                };
                nic.send(&buf[..unwritten])
            }
            _ => unimplemented!(),
        }
    }
}
