use archway_proto::archway::cwica::v1::{MsgRegisterInterchainAccount, MsgRegisterInterchainAccountResponse, MsgSendTx, MsgSendTxResponse};
use test_tube::{fn_execute, Module, Runner};

pub struct CwIca<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for CwIca<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> CwIca<'a, R>
    where
        R: Runner<'a>,
{
    fn_execute! {
        pub register_interchain_account: MsgRegisterInterchainAccount["/archway.cwica.v1.MsgRegisterInterchainAccount"] => MsgRegisterInterchainAccountResponse
    }
    fn_execute! {
        pub send_tx: MsgSendTx["/archway.cwica.v1.MsgSendTx"] => MsgSendTxResponse
    }
}