use crate::{coin, ADDR_PREFIX, FAUCET_MNEMONIC};
use archway_rpc::utils::Fee;
use archway_rpc::{
    from_mnemonic_phrase, from_mnemonic_random, Account, ArchwayClient, Bank, CosmosDerivationPath,
    HttpClient, Id, Result,
};

pub struct ChainClient {
    pub client: ArchwayClient,
    pub id: Id,
}

impl ChainClient {
    /// Create a new chain client
    pub async fn new(id: Id, path: &str) -> Result<Self> {
        let mut client = ArchwayClient::new(HttpClient::new(path)?);
        client.fee = Fee::from_amount_and_gas(coin(1).into(), 10000000000u64);

        Ok(Self { client, id })
    }

    /// Build and return the faucet account
    pub async fn faucet_account(&self) -> Result<Account<'_>> {
        Account::new(
            &self.client,
            ADDR_PREFIX,
            from_mnemonic_phrase(FAUCET_MNEMONIC, CosmosDerivationPath::default())?,
            &self.id,
        )
        .await
    }

    /// Create a new account and fund it with native tokens
    pub async fn new_account(&self, amount: u128) -> Result<Account<'_>> {
        let mut acc = Account::new(
            &self.client,
            ADDR_PREFIX,
            from_mnemonic_random(CosmosDerivationPath::default())?.0,
            &self.id,
        )
        .await?;

        // Fund account if there is an amount
        if amount > 0 {
            self.fund_account(&acc, amount).await?;
            // This account did not exist before funding, so we need to refresh its info
            acc.reload(&self.client).await?;
        }

        Ok(acc)
    }

    /// Send tokens from faucet to the given address
    pub async fn fund_account<'a>(&self, acc: &Account<'a>, amount: u128) -> Result<()> {
        let mut faucet = self.faucet_account().await?;

        self.client
            .send(
                &faucet,
                acc.prefixed_pubkey().unwrap().to_string(),
                vec![coin(amount)],
            )?
            .poll(&mut faucet)
            .await?;
        Ok(())
    }
}
