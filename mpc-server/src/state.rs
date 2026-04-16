
#[derive(Clone)]
pub struct MpcState{
    pub node_id:u16,
    pub hmac_secret:String,
    pub aes_secret_key:String,
}