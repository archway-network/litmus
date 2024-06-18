use archway_proto::archway::callback::v1::{MsgCancelCallback, MsgCancelCallbackResponse, MsgRequestCallback, MsgRequestCallbackResponse, QueryCallbacksRequest, QueryCallbacksResponse, QueryEstimateCallbackFeesRequest, QueryEstimateCallbackFeesResponse};
use test_tube::{fn_execute, fn_query, Module, Runner};

pub struct Callback<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Callback<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Callback<'a, R>
    where
        R: Runner<'a>,
{
    fn_execute! {
        pub request_callback: MsgRequestCallback["/archway.callback.v1.MsgRequestCallback"] => MsgRequestCallbackResponse
    }

    fn_execute! {
        pub cancel_callback: MsgCancelCallback["/archway.callback.v1.MsgCancelCallback"] => MsgCancelCallbackResponse
    }
    
    fn_query! {
        pub query_estimate_callback_fees ["/archway.callback.v1.Query/EstimateCallbackFees"]: QueryEstimateCallbackFeesRequest => QueryEstimateCallbackFeesResponse
    }

    fn_query! {
        pub query_callbacks ["/archway.callback.v1.Query/Callbacks"]: QueryCallbacksRequest => QueryCallbacksResponse
    }
}