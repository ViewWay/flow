use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;
use std::sync::RwLock;
use serde_json::Value;

/// 加密服务（密钥管理）
pub struct CryptoService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    secret: Arc<RwLock<String>>,
}

impl CryptoService {
    /// 从密钥创建加密服务
    pub fn from_secret(secret: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        Ok(Self {
            encoding_key,
            decoding_key,
            secret: Arc::new(RwLock::new(secret.to_string())),
        })
    }

    /// 获取编码密钥
    pub fn encoding_key(&self) -> &EncodingKey {
        &self.encoding_key
    }

    /// 获取解码密钥
    pub fn decoding_key(&self) -> &DecodingKey {
        &self.decoding_key
    }

    /// 获取JWK（简化实现，返回JSON格式）
    pub fn get_jwk(&self) -> Value {
        serde_json::json!({
            "kty": "oct",
            "use": "sig",
            "alg": "HS256",
            "kid": "default"
        })
    }

    /// 获取JWK Set（用于OAuth2）
    pub fn get_jwk_set(&self) -> Value {
        serde_json::json!({
            "keys": [self.get_jwk()]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_service_creation() {
        let service = CryptoService::from_secret("test_secret").unwrap();
        let jwk = service.get_jwk();
        assert_eq!(jwk["alg"], "HS256");
    }
}

