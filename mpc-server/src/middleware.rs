
use actix_web::HttpResponse;
use actix_web::web::Bytes;
use hmac::{Hmac,Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn is_authentic(req:&HttpResponse,body_bytes:&Bytes,secret_key:&str) ->bool{

    let provided_signature = match req.headers().get("X-Signature").and_then(|v| v.to_str().ok()){
        Some(sig) => match sig.to_str(){
            Ok(s) => s,
            Err(_) => return false,
        }
    };

    let timestamp_str = match req.headers().get("X-Timestamp").and_then(|v| v.to_str()){
        Some(s) => s,
        None => return false,
    };


    let now = Utc::now().timestamp();
    let ts:i64 = timestamp_str.parse().unwrap_or(0);
    if (now - ts).abs() > 300 {
        return false
    }


    let mut mac = match HmacSha256::new_from_slice(secret_key.as_bytes()){
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body_bytes);
    mac.update(timestamp_str.as_bytes());
    let result = mac.finalize().into_bytes();
    let calculated_signature = format!("{:x}", result);
    calculated_signature == provided_signature
}
