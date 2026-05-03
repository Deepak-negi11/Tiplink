-- Custom enum types
CREATE TYPE tx_type AS ENUM ('send', 'receive', 'swap', 'claim', 'escrow');
CREATE TYPE link_status AS ENUM ('active', 'claimed', 'expired', 'cancelled');
CREATE TYPE swap_status AS ENUM ('pending', 'confirmed', 'failed');

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    balance NUMERIC NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    public_key VARCHAR(44) NOT NULL UNIQUE
);

-- Sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token TEXT NOT NULL,
    device_info TEXT,
    ip_address INET,
    revoked_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Balances table
CREATE TABLE balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amount BIGINT NOT NULL DEFAULT 0,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_mint VARCHAR(44) NOT NULL,
    token_symbol VARCHAR(44) NOT NULL,
    locked BIGINT NOT NULL DEFAULT 0,
    available BIGINT NOT NULL DEFAULT 0,
    decimals SMALLINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount BIGINT NOT NULL,
    token_mint VARCHAR(44) NOT NULL,
    token_symbol VARCHAR(44) NOT NULL,
    tx_hash VARCHAR(88) NOT NULL,
    tx_type tx_type NOT NULL,
    from_address VARCHAR(44) NOT NULL,
    to_address VARCHAR(44) NOT NULL,
    slot BIGINT NOT NULL,
    block_time TIMESTAMPTZ NOT NULL
);

-- Transaction intents table
CREATE TABLE transaction_intents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    intent_message TEXT NOT NULL,
    intent_signature TEXT NOT NULL,
    unsigned_payload TEXT,
    status VARCHAR(50),
    final_tx_hash TEXT,
    created_at TIMESTAMP
);

-- Payment links table
CREATE TABLE payment_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    escrow_pda VARCHAR(44) NOT NULL,
    claim_hash VARCHAR(64) NOT NULL,
    token_mint VARCHAR(44) NOT NULL,
    amount BIGINT NOT NULL,
    recipient_email VARCHAR(255),
    recipient_phone VARCHAR(20),
    status link_status NOT NULL DEFAULT 'active',
    claimed_by UUID REFERENCES users(id),
    claim_tx_hash VARCHAR(88),
    expires_at TIMESTAMPTZ NOT NULL,
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    claimed_at TIMESTAMPTZ
);

-- Swap history table
CREATE TABLE swap_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    input_mint VARCHAR(44) NOT NULL,
    output_mint VARCHAR(44) NOT NULL,
    output_amount BIGINT NOT NULL,
    input_amount BIGINT NOT NULL,
    fee_amount BIGINT NOT NULL DEFAULT 0,
    price_impact NUMERIC NOT NULL DEFAULT 0,
    tx_hash VARCHAR(88) NOT NULL,
    status swap_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ,
    requested_slippage_bps INTEGER NOT NULL DEFAULT 50
);

-- Indexes
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_balances_user_id ON balances(user_id);
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transaction_intents_user_id ON transaction_intents(user_id);
CREATE INDEX idx_payment_links_creator_id ON payment_links(creator_id);
CREATE INDEX idx_swap_history_user_id ON swap_history(user_id);
CREATE UNIQUE INDEX idx_balances_user_token ON balances(user_id, token_mint);
