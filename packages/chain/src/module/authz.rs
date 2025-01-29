use archway_proto::cosmos::authz::v1beta1::{
    GenericAuthorization, Grant, MsgExec, MsgExecResponse, MsgGrant, MsgGrantResponse,
};
use archway_proto::cosmos::bank::v1beta1::SendAuthorization;
use archway_proto::cosmos::staking::v1beta1::stake_authorization::Policy;
use archway_proto::cosmos::staking::v1beta1::{AuthorizationType, StakeAuthorization};
use archway_proto::tendermint::google::protobuf::{Any, Timestamp};
use cosmwasm_std::Coin;
use prost::Name;
use test_tube::cosmrs::tx::MessageExt;
use test_tube::{
    fn_execute, Account, Module, Runner, RunnerError, RunnerExecuteResult, SigningAccount,
};

pub struct Authz<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Authz<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

impl<'a, R> Authz<'a, R>
where
    R: Runner<'a>,
{
    fn_execute! {
        pub exec: MsgExec["/cosmos.authz.v1beta1.MsgExec"] => MsgExecResponse
    }

    fn_execute! {
        pub _grant: MsgGrant["/cosmos.authz.v1beta1.MsgGrant"] => MsgGrantResponse
    }

    pub fn grant<T>(
        &self,
        signer: &SigningAccount,
        grantee: impl Into<String>,
        msg: T,
        expiration: Option<Timestamp>,
    ) -> RunnerExecuteResult<MsgGrantResponse>
    where
        T: Name + MessageExt,
    {
        let granter = signer.address();
        let msg = MsgGrant {
            granter,
            grantee: grantee.into(),
            grant: Some(Grant {
                authorization: Some(Any {
                    type_url: T::type_url(),
                    value: msg
                        .to_bytes()
                        .map_err(|e| RunnerError::EncodeError(e.into()))?,
                }),
                expiration,
            }),
        };
        self._grant(msg, signer)
    }

    pub fn grant_generic(
        &self,
        signer: &SigningAccount,
        grantee: impl Into<String>,
        authorized_msg: String,
        expiration: Option<Timestamp>,
    ) -> RunnerExecuteResult<MsgGrantResponse> {
        self.grant(
            signer,
            grantee,
            GenericAuthorization {
                msg: authorized_msg,
            },
            expiration,
        )
    }

    pub fn grant_stake(
        &self,
        signer: &SigningAccount,
        grantee: impl Into<String>,
        auth_type: AuthorizationType,
        max_tokens: Option<Coin>,
        validators: Option<Policy>,
        expiration: Option<Timestamp>,
    ) -> RunnerExecuteResult<MsgGrantResponse> {
        self.grant(
            signer,
            grantee,
            StakeAuthorization {
                max_tokens: max_tokens.map(|c| coin_to_proto_coin(&c)),
                authorization_type: auth_type.into(),
                validators,
            },
            expiration,
        )
    }

    pub fn grant_send(
        &self,
        signer: &SigningAccount,
        grantee: impl Into<String>,
        allow_list: Vec<String>,
        spend_limit: Vec<Coin>,
        expiration: Option<Timestamp>,
    ) -> RunnerExecuteResult<MsgGrantResponse> {
        self.grant(
            signer,
            grantee,
            SendAuthorization {
                spend_limit: spend_limit.iter().map(|c| coin_to_proto_coin(c)).collect(),
                allow_list,
            },
            expiration,
        )
    }
}

fn coin_to_proto_coin(coin: &Coin) -> archway_proto::cosmos::base::v1beta1::Coin {
    archway_proto::cosmos::base::v1beta1::Coin {
        denom: coin.denom.clone(),
        amount: coin.amount.to_string(),
    }
}
