use archway_proto::cosmos::base::query::v1beta1::PageRequest;
use archway_proto::cosmos::staking::v1beta1::{
    MsgDelegate, MsgDelegateResponse, QueryDelegationRequest, QueryDelegationResponse,
    QueryValidatorsRequest, QueryValidatorsResponse,
};
use test_tube::{fn_execute, fn_query, Module, Runner, RunnerResult};
pub struct Staking<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Staking<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Staking<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub delegate: MsgDelegate["/cosmos.staking.v1beta1.MsgDelegate"] => MsgDelegateResponse
    }

    fn_query! {
        pub query_validators ["/cosmos.staking.v1beta1.Query/Validators"]: QueryValidatorsRequest => QueryValidatorsResponse
    }

    fn_query! {
        pub query_delegation ["/cosmos.staking.v1beta1.Query/Delegation"]: QueryDelegationRequest => QueryDelegationResponse
    }

    pub fn validators(
        &self,
        pagination: Option<PageRequest>,
        status: Option<String>,
    ) -> RunnerResult<QueryValidatorsResponse> {
        self.query_validators(&QueryValidatorsRequest {
            status: status.unwrap_or_default(),
            pagination,
        })
    }

    pub fn delegation(
        &self,
        delegator: impl Into<String>,
        validator: impl Into<String>,
    ) -> RunnerResult<QueryDelegationResponse> {
        self.query_delegation(&QueryDelegationRequest {
            delegator_addr: delegator.into(),
            validator_addr: validator.into(),
        })
    }
}
