use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT Claims结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (username)
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
    pub iss: String, // issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pat_name: Option<String>, // Personal Access Token name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>, // JWT ID
}

impl Claims {
    pub fn new(sub: String, issuer: String, expiration_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        
        Self {
            sub,
            exp: now + expiration_seconds as usize,
            iat: now,
            iss: issuer,
            pat_name: None,
            jti: None,
        }
    }

    pub fn with_pat(mut self, pat_name: String, jti: String) -> Self {
        self.pat_name = Some(pat_name);
        self.jti = Some(jti);
        self
    }
}

/// JWT服务
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    expiration: u64,
}

impl JwtService {
    /// 创建新的JWT服务
    pub fn new(secret: &str, issuer: String, expiration: u64) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        Ok(Self {
            encoding_key,
            decoding_key,
            issuer,
            expiration,
        })
    }

    /// 生成JWT令牌
    pub fn generate(&self, username: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let claims = Claims::new(username, self.issuer.clone(), self.expiration);
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| format!("JWT encode error: {}", e))?;
        Ok(token)
    }

    /// 生成PAT令牌
    pub fn generate_pat(&self, username: String, pat_name: String, jti: String) 
        -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let claims = Claims::new(username, self.issuer.clone(), self.expiration)
            .with_pat(pat_name, jti);
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| format!("JWT encode error: {}", e))?;
        Ok(token)
    }

    /// 验证JWT令牌
    pub fn verify(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error + Send + Sync>> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| format!("JWT decode error: {}", e))?;
        
        Ok(token_data.claims)
    }

    /// 获取过期时间（秒）
    pub fn expiration(&self) -> u64 {
        self.expiration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generate_and_verify() {
        let service = JwtService::new("test_secret", "test_issuer".to_string(), 3600).unwrap();
        
        let token = service.generate("test_user".to_string()).unwrap();
        assert!(!token.is_empty());
        
        let claims = service.verify(&token).unwrap();
        assert_eq!(claims.sub, "test_user");
        assert_eq!(claims.iss, "test_issuer");
    }

    #[test]
    fn test_jwt_pat() {
        let service = JwtService::new("test_secret", "test_issuer".to_string(), 3600).unwrap();
        
        let token = service.generate_pat(
            "test_user".to_string(),
            "test-pat".to_string(),
            "test-jti".to_string(),
        ).unwrap();
        
        let claims = service.verify(&token).unwrap();
        assert_eq!(claims.pat_name, Some("test-pat".to_string()));
        assert_eq!(claims.jti, Some("test-jti".to_string()));
    }
}

