#[derive(Copy, Clone)]
/// Used for keeping information for the docker exposed chains
pub(crate) struct ChainInfo<'a> {
    pub(crate) id: &'a str,
    pub(crate) rpc_port: usize,
}

pub(crate) const CHAIN1: ChainInfo<'static> = ChainInfo {
    id: "archway-1",
    rpc_port: 27010,
};

pub(crate) const CHAIN2: ChainInfo<'static> = ChainInfo {
    id: "archway-2",
    rpc_port: 27020,
};
