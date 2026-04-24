pub mod deposit;
pub mod link;
pub mod swap;
pub mod withdraw;

use crate::db::pool::DbPool;
use crate::filters::accounts::TrackedAccounts;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction;

pub fn process_transaction(
    pool: &DbPool,
    accounts: &TrackedAccounts,
    update: SubscribeUpdateTransaction,
) {
    let tx = match update.transaction {
        Some(t) => t,
        None => return,
    };

    let meta = match tx.meta {
        Some(m) => m,
        None => return,
    };

    // Ignore failed transactions
    if meta.err.is_some() {
        return;
    }

    let solana_tx = match tx.transaction {
        Some(t) => t,
        None => return,
    };

    let message = match solana_tx.message {
        Some(m) => m,
        None => return,
    };

    // Extract signature
    let sig_bytes = tx.signature.clone();
    if sig_bytes.is_empty() {
        return;
    }
    let sig = bs58::encode(&sig_bytes).into_string();

    // Map account indices to actual base58 public keys
    let account_keys: Vec<String> = message
        .account_keys
        .iter()
        .map(|k| bs58::encode(k).into_string())
        .collect();

    // Calculate SOL changes
    for (i, pubkey) in account_keys.iter().enumerate() {
        // Did we track this user?
        if let Some(user_id) = accounts.get_user_id(pubkey) {
            let pre_bal = meta.pre_balances.get(i).copied().unwrap_or(0);
            let post_bal = meta.post_balances.get(i).copied().unwrap_or(0);

            let diff = post_bal as i128 - pre_bal as i128;
            
            // Handle SOL transfers
            if diff > 0 {
                // Deposit
                tracing::info!("Detected SOL deposit of {} lamports to {} (tx: {})", diff, pubkey, sig);
                deposit::handle_deposit(
                    pool,
                    user_id,
                    diff as i64,
                    crate::filters::program::NATIVE_SOL_MINT, // "So11...112"
                    "SOL",
                    9, // SOL decimals
                    &sig,
                    "Unknown Sender", // Fully resolving sender requires parsing instructions
                    pubkey,
                    update.slot as i64,
                );
            } else if diff < 0 {
                // Withdrawal or fee
                // Note: We might want to ignore tiny diffs that are just gas fees, but for now we track them.
                tracing::info!("Detected SOL withdrawal of {} lamports from {} (tx: {})", diff.abs(), pubkey, sig);
                withdraw::handle_withdraw(
                    pool,
                    user_id,
                    diff.abs() as i64,
                    crate::filters::program::NATIVE_SOL_MINT,
                    "SOL",
                    &sig,
                    pubkey,
                    "Unknown Destination",
                    update.slot as i64,
                );
            }
        }
    }

    // Calculate SPL Token changes
    // We use a Map to group pre and post balances by account_index
    let mut token_diffs = std::collections::HashMap::new();

    for pre in &meta.pre_token_balances {
        token_diffs.insert(pre.account_index, (Some(pre.clone()), None));
    }
    for post in &meta.post_token_balances {
        token_diffs.entry(post.account_index).and_modify(|e| e.1 = Some(post.clone())).or_insert((None, Some(post.clone())));
    }

    for (_account_idx, (pre, post)) in token_diffs {
        let owner = if let Some(ref p) = post {
            p.owner.clone()
        } else if let Some(ref p) = pre {
            p.owner.clone()
        } else {
            continue;
        };

        if let Some(user_id) = accounts.get_user_id(&owner) {
            let pre_amt: i128 = pre.as_ref().and_then(|p| p.ui_token_amount.as_ref()).map(|amt| amt.amount.parse().unwrap_or(0)).unwrap_or(0);
            let post_amt: i128 = post.as_ref().and_then(|p| p.ui_token_amount.as_ref()).map(|amt| amt.amount.parse().unwrap_or(0)).unwrap_or(0);
            
            let diff = post_amt - pre_amt;
            let mint = post.as_ref().map(|p| p.mint.clone()).unwrap_or_else(|| pre.as_ref().unwrap().mint.clone());
            let decimals = post.as_ref().and_then(|p| p.ui_token_amount.as_ref()).map(|amt| amt.decimals).unwrap_or(0);

            if diff > 0 {
                 deposit::handle_deposit(
                    pool,
                    user_id,
                    diff as i64,
                    &mint,
                    "SPL", // Would need a token registry to look up actual symbol
                    decimals as i16,
                    &sig,
                    "Unknown Sender",
                    &owner,
                    update.slot as i64,
                );
            } else if diff < 0 {
                 withdraw::handle_withdraw(
                    pool,
                    user_id,
                    diff.abs() as i64,
                    &mint,
                    "SPL",
                    &sig,
                    &owner,
                    "Unknown Destination",
                    update.slot as i64,
                );
            }
        }
    }

    // TODO: Detect Jupiter / Swap program invocations via instructions
    // TODO: Detect TipLink program invocations via instructions
}
