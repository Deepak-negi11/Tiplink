CREATE TABLE mpc_vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Users live in the main database; MPC nodes store only encrypted shares.
    -- A PostgreSQL foreign key cannot enforce a relationship across databases.
    user_id UUID NOT NULL,
    node_id INTEGER NOT NULL,
    key_package TEXT NOT NULL,
    pubkey_package TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, node_id)
);
