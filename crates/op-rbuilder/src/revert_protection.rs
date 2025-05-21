use crate::primitives::bundle::{Bundle, MAX_BLOCK_RANGE_BLOCKS};
use crate::tx::{FBPoolTransaction, FBPooledTransaction};
use alloy_primitives::B256;
use alloy_rpc_types_eth::erc4337::TransactionConditional;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
};
use reth::rpc::result::rpc_err;
use reth_optimism_txpool::{conditional::MaybeConditionalTransaction, OpPooledTransaction};
use reth_provider::StateProviderFactory;
use reth_rpc_eth_types::utils::recover_raw_transaction;
use reth_transaction_pool::{PoolTransaction, TransactionOrigin, TransactionPool};

// Namespace overrides for revert protection support
#[cfg_attr(not(test), rpc(server, namespace = "eth"))]
#[cfg_attr(test, rpc(server, client, namespace = "eth"))]
pub trait EthApiOverride {
    #[method(name = "sendRawTransactionRevert")]
    async fn send_raw_transaction_revert(&self, tx: Bundle) -> RpcResult<B256>;
}

pub struct RevertProtectionExt<Pool, Provider> {
    pool: Pool,
    provider: Provider,
}

impl<Pool, Provider> RevertProtectionExt<Pool, Provider> {
    pub fn new(pool: Pool, provider: Provider) -> Self {
        Self { pool, provider }
    }
}

impl Bundle {
    fn conditional(&self) -> TransactionConditional {
        TransactionConditional {
            block_number_min: None,
            block_number_max: self.block_number_max,
            known_accounts: Default::default(),
            timestamp_max: None,
            timestamp_min: None,
        }
    }
}

#[async_trait]
impl<Pool, Provider> EthApiOverrideServer for RevertProtectionExt<Pool, Provider>
where
    Pool: TransactionPool<Transaction = FBPooledTransaction> + Clone + 'static,
    Provider: StateProviderFactory + Send + Sync + Clone + 'static,
{
    async fn send_raw_transaction_revert(&self, mut bundle: Bundle) -> RpcResult<B256> {
        let last_block_number = self.provider.best_block_number().unwrap(); // FIXME: do not unwrap

        if let Some(block_number_max) = bundle.block_number_max {
            // The max block cannot be a past block
            if block_number_max <= last_block_number {
                return Err(rpc_err(
                    jsonrpsee_types::error::INVALID_PARAMS_CODE,
                    "block_number_max is a past block",
                    None,
                ));
            }

            // Validate that it is not greater than the max_block_range
            if block_number_max > last_block_number + MAX_BLOCK_RANGE_BLOCKS {
                return Err(rpc_err(
                    jsonrpsee_types::error::INVALID_PARAMS_CODE,
                    "block_number_max is too high",
                    None,
                ));
            }
        } else {
            // If no upper bound is set, use the maximum block range
            bundle.block_number_max = Some(last_block_number + MAX_BLOCK_RANGE_BLOCKS);
        }

        let recovered = recover_raw_transaction(&bundle.transaction)?;
        let mut pool_transaction: FBPooledTransaction =
            OpPooledTransaction::from_pooled(recovered).into();

        println!("conditional: {:?}", bundle.conditional());

        pool_transaction.set_exclude_reverting_txs(true);
        pool_transaction.set_conditional(bundle.conditional());

        let hash = self
            .pool
            .add_transaction(TransactionOrigin::Local, pool_transaction)
            .await
            .unwrap(); // TODO: FIX THIS

        Ok(hash)
    }
}
