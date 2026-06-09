ALTER TABLE balances ALTER COLUMN token_mint TYPE VARCHAR(64);
ALTER TABLE transactions ALTER COLUMN token_mint TYPE VARCHAR(64);
ALTER TABLE payment_links ALTER COLUMN token_mint TYPE VARCHAR(64);
ALTER TABLE swap_history ALTER COLUMN input_mint TYPE VARCHAR(64);
ALTER TABLE swap_history ALTER COLUMN output_mint TYPE VARCHAR(64);

UPDATE balances
SET token_mint = 'devnet_So11111111111111111111111111111111111111112'
WHERE token_mint = 'devnet_So1111111111111111111111111111111111111';

UPDATE balances
SET token_mint = 'devnet_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'
WHERE token_mint = 'devnet_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZw';
