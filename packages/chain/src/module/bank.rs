use archway_proto::cosmos::bank::v1beta1::{
    MsgSend, MsgSendResponse, QueryAllBalancesRequest, QueryAllBalancesResponse,
    QueryBalanceRequest, QueryBalanceResponse, QueryDenomsMetadataRequest,
    QueryDenomsMetadataResponse, QueryTotalSupplyRequest, QueryTotalSupplyResponse,
};
use test_tube::{fn_execute, fn_query, Module, Runner, RunnerResult};

pub struct Bank<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Bank<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Bank<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub send: MsgSend["/cosmos.bank.v1beta1.MsgSend"] => MsgSendResponse
    }

    fn_query! {
        pub query_balance ["/cosmos.bank.v1beta1.Query/Balance"]: QueryBalanceRequest => QueryBalanceResponse
    }

    fn_query! {
        pub query_all_balances ["/cosmos.bank.v1beta1.Query/AllBalances"]: QueryAllBalancesRequest => QueryAllBalancesResponse
    }

    fn_query! {
        pub query_total_supply ["/cosmos.bank.v1beta1.Query/TotalSupply"]: QueryTotalSupplyRequest => QueryTotalSupplyResponse
    }

    fn_query! {
        pub query_denoms_metadata ["/cosmos.bank.v1beta1.Query/DenomsMetadata"]: QueryDenomsMetadataRequest => QueryDenomsMetadataResponse
    }

    pub fn balance(
        &self,
        address: impl Into<String>,
        denom: impl Into<String>,
    ) -> RunnerResult<QueryBalanceResponse> {
        self.query_balance(&QueryBalanceRequest {
            address: address.into(),
            denom: denom.into(),
        })
    }
}
