use archway_proto::cosmos::base::v1beta1::Coin as ProtoCoin;
use cosmwasm_std::{Coin, Uint128};

pub fn from_legacy(coin: &cosmwasm_std_legacy::Coin) -> Coin {
    Coin {
        denom: coin.denom.clone(),
        amount: Uint128::from(coin.amount.u128()),
    }
}

pub fn to_proto(coin: &Coin) -> ProtoCoin {
    ProtoCoin {
        denom: coin.denom.clone(),
        amount: coin.amount.to_string(),
    }
}
