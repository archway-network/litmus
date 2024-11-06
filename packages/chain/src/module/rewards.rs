use archway_proto::archway::rewards::v1::{
    MsgSetContractMetadata, MsgSetContractMetadataResponse, MsgSetFlatFee, MsgSetFlatFeeResponse,
    MsgWithdrawRewards, MsgWithdrawRewardsResponse, QueryBlockRewardsTrackingRequest,
    QueryBlockRewardsTrackingResponse, QueryContractMetadataRequest, QueryContractMetadataResponse,
    QueryEstimateTxFeesRequest, QueryEstimateTxFeesResponse, QueryFlatFeeRequest,
    QueryFlatFeeResponse, QueryOutstandingRewardsRequest, QueryOutstandingRewardsResponse,
    QueryRewardsPoolRequest, QueryRewardsPoolResponse, QueryRewardsRecordsRequest,
    QueryRewardsRecordsResponse,
};
use test_tube::{fn_execute, fn_query, Module, Runner};

pub struct Rewards<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Rewards<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Rewards<'a, R>
where
    R: Runner<'a>,
{
    fn_query! {
        pub query_contract_metadata ["/archway.rewards.v1.Query/ContractMetadata"]: QueryContractMetadataRequest => QueryContractMetadataResponse
    }
    fn_query! {
        pub query_block_rewards_tracking ["/archway.rewards.v1.Query/BlockRewardsTracking"]: QueryBlockRewardsTrackingRequest => QueryBlockRewardsTrackingResponse
    }
    fn_query! {
        pub query_rewards_pool ["/archway.rewards.v1.Query/RewardsPool"]: QueryRewardsPoolRequest => QueryRewardsPoolResponse
    }
    fn_query! {
        pub query_estimate_tx_fees ["/archway.rewards.v1.Query/EstimateTxFees"]: QueryEstimateTxFeesRequest => QueryEstimateTxFeesResponse
    }
    fn_query! {
        pub query_rewards_records ["/archway.rewards.v1.Query/RewardsRecords"]: QueryRewardsRecordsRequest => QueryRewardsRecordsResponse
    }
    fn_query! {
        pub query_outstanding_rewards ["/archway.rewards.v1.Query/OutstandingRewards"]: QueryOutstandingRewardsRequest => QueryOutstandingRewardsResponse
    }
    fn_query! {
        pub query_flat_fee ["/archway.rewards.v1.Query/FlatFee"]: QueryFlatFeeRequest => QueryFlatFeeResponse
    }
    fn_execute! {
        pub set_contract_metadata: MsgSetContractMetadata["/archway.rewards.v1.MsgSetContractMetadata"] => MsgSetContractMetadataResponse
    }
    fn_execute! {
        pub withdraw_rewards: MsgWithdrawRewards["/archway.rewards.v1.MsgWithdrawRewards"] => MsgWithdrawRewardsResponse
    }
    fn_execute! {
        pub set_flat_fee: MsgSetFlatFee["/archway.rewards.v1.MsgSetFlatFee"] => MsgSetFlatFeeResponse
    }
}
