use frost_ed25519 as frost;
use crate::error::MpcError;

pub fn parse_identifier(id_str: &str) -> Result<frost::Identifier, MpcError> {
    if let Ok(num) = id_str.parse::<u16>() {
        return num.try_into()
            .map_err(|_| MpcError::BadRequest(format!("Invalid FROST identifier from u16: {}", id_str)));
    }
    let quoted = format!("\"{}\"", id_str);
    serde_json::from_str::<frost::Identifier>(&quoted)
        .map_err(|e| MpcError::BadRequest(format!("Cannot parse FROST identifier '{}': {}", id_str, e)))
}
