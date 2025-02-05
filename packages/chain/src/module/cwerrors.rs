use archway_proto::archway::cwerrors::v1::{QueryErrorsRequest, QueryErrorsResponse};
use test_tube::{fn_query, Module, Runner};

pub struct CwErrors<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for CwErrors<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> CwErrors<'a, R>
where
    R: Runner<'a>,
{
    fn_query! {
        pub errors ["/archway.cwerrors.v1.Query/Errors"]: QueryErrorsRequest => QueryErrorsResponse
    }
}
