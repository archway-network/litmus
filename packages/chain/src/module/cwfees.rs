use archway_proto::archway::cwfees::v1::{
    IsGrantingContractRequest, IsGrantingContractResponse, MsgRegisterAsGranter,
    MsgRegisterAsGranterResponse, MsgUnregisterAsGranter, MsgUnregisterAsGranterResponse,
};
use test_tube::{fn_execute, fn_query, Module, Runner};

pub struct CwFees<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for CwFees<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> CwFees<'a, R>
where
    R: Runner<'a>,
{
    fn_query! {
        pub is_granting_contract ["/archway.cwfees.v1.Query/IsGrantingContract"]: IsGrantingContractRequest => IsGrantingContractResponse
    }
    fn_execute! {
        pub register_as_granter: MsgRegisterAsGranter["/archway.cwfees.v1.MsgRegisterAsGranter"] => MsgRegisterAsGranterResponse
    }
    fn_execute! {
        pub unregister_as_granter: MsgUnregisterAsGranter["/archway.cwfees.v1.MsgUnregisterAsGranter"] => MsgUnregisterAsGranterResponse
    }
}
