pub const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
pub const SPL_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const NATIVE_SOL_MINT: &str = "So11111111111111111111111111111111111111112";

pub fn is_system_transfer(program_id: &str) -> bool {
    program_id == SYSTEM_PROGRAM
}

pub fn is_spl_transfer(program_id: &str) -> bool {
    program_id == SPL_TOKEN_PROGRAM
}
