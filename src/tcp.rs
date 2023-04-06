pub struct State {

}

impl Default for State {
    fn default() -> Self {
        Self {  }
    }
}

impl State {
    pub fn on_packet<'a>(&mut self, iph: etherparse::Ipv4HeaderSlice<'a>, tcph: etherparse::TcpHeaderSlice<'a>, data: &'a [u8]) {
        eprintln!(
            "{}:{} - {}:{} {}b",
            iph.source_addr(),
            tcph.source_port(),
            iph.destination_addr(),
            tcph.destination_port(),
            data.len(),
        );
    }
}