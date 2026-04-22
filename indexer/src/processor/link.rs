pub fn handle_claim() {
    tracing::debug!("Payment link claim intersection detected. (Full parsing logic deferred)");
    // TODO:
    // 1. Extract the escrow PDA address
    // 2. Identify the user who claimed it (the destination wallet)
    // 3. Call Balance::finalize_claim in Postgres
    // 4. Update payment_links table status to 'Claimed'
}
