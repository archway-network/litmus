use archway_proto::cosmos::distribution::v1beta1::{
    QueryDelegationRewardsRequest, QueryDelegationRewardsResponse,
};
use archway_proto::cosmos::staking::v1beta1::{QueryDelegationRequest, QueryDelegationResponse};
use test_tube::{fn_query, Module, Runner, RunnerResult};

pub struct Distribution<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Distribution<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Distribution<'a, R>
where
    R: Runner<'a>,
{
    fn_query! {
        pub query_delegation_rewards ["/cosmos.distribution.v1beta1.Query/DelegationRewards"]: QueryDelegationRewardsRequest => QueryDelegationRewardsResponse
    }

    fn_query! {
        pub query_delegation ["/cosmos.distribution.v1beta1.Query/Delegation"]: QueryDelegationRequest => QueryDelegationResponse
    }

    pub fn delegation_rewards(
        &self,
        delegator: impl Into<String>,
        validator: impl Into<String>,
    ) -> RunnerResult<QueryDelegationRewardsResponse> {
        self.query_delegation_rewards(&QueryDelegationRewardsRequest {
            delegator_address: delegator.into(),
            validator_address: validator.into(),
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
