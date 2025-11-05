use async_trait::async_trait;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use bcrypt::{hash, verify, DEFAULT_COST};

/// 密码加密算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordAlgorithm {
    Bcrypt,
    Argon2,
}

/// 密码服务trait
#[async_trait]
pub trait PasswordService: Send + Sync {
    /// 加密密码
    async fn hash(&self, password: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 验证密码
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取使用的加密算法
    fn algorithm(&self) -> PasswordAlgorithm;
}

/// 默认密码服务实现
pub struct DefaultPasswordService {
    algorithm: PasswordAlgorithm,
    bcrypt_cost: u32,
}

impl DefaultPasswordService {
    pub fn new(algorithm: PasswordAlgorithm) -> Self {
        Self {
            algorithm,
            bcrypt_cost: DEFAULT_COST,
        }
    }

    pub fn with_bcrypt_cost(mut self, cost: u32) -> Self {
        self.bcrypt_cost = cost;
        self
    }
}

#[async_trait]
impl PasswordService for DefaultPasswordService {
    async fn hash(&self, password: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.algorithm {
            PasswordAlgorithm::Bcrypt => {
                hash(password, self.bcrypt_cost)
                    .map_err(|e| format!("Bcrypt hash error: {}", e).into())
            }
            PasswordAlgorithm::Argon2 => {
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password_hash = argon2.hash_password(password.as_bytes(), &salt)
                    .map_err(|e| format!("Argon2 hash error: {}", e))?;
                Ok(password_hash.to_string())
            }
        }
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self.algorithm {
            PasswordAlgorithm::Bcrypt => {
                verify(password, hash)
                    .map_err(|e| format!("Bcrypt verify error: {}", e).into())
            }
            PasswordAlgorithm::Argon2 => {
                let parsed_hash = PasswordHash::new(hash)
                    .map_err(|e| format!("Argon2 parse hash error: {}", e))?;
                let argon2 = Argon2::default();
                match argon2.verify_password(password.as_bytes(), &parsed_hash) {
                    Ok(()) => Ok(true),
                    Err(argon2::password_hash::Error::Password) => Ok(false),
                    Err(e) => Err(format!("Argon2 verify error: {}", e).into()),
                }
            }
        }
    }

    fn algorithm(&self) -> PasswordAlgorithm {
        self.algorithm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bcrypt_hash_and_verify() {
        let service = DefaultPasswordService::new(PasswordAlgorithm::Bcrypt);
        let password = "test_password_123";
        
        let hash = service.hash(password).await.unwrap();
        assert!(!hash.is_empty());
        
        let verified = service.verify(password, &hash).await.unwrap();
        assert!(verified);
        
        let wrong_password = "wrong_password";
        let verified_wrong = service.verify(wrong_password, &hash).await.unwrap();
        assert!(!verified_wrong);
    }

    #[tokio::test]
    async fn test_argon2_hash_and_verify() {
        let service = DefaultPasswordService::new(PasswordAlgorithm::Argon2);
        let password = "test_password_123";
        
        let hash = service.hash(password).await.unwrap();
        assert!(!hash.is_empty());
        
        let verified = service.verify(password, &hash).await.unwrap();
        assert!(verified);
        
        let wrong_password = "wrong_password";
        let verified_wrong = service.verify(wrong_password, &hash).await.unwrap();
        assert!(!verified_wrong);
    }
}

