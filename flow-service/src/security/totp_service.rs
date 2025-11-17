use async_trait::async_trait;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sha2::{Sha256, Digest};
use hex;

/// TOTP认证服务trait
#[async_trait]
pub trait TotpAuthService: Send + Sync {
    /// 验证TOTP代码
    /// 
    /// # 参数
    /// - `raw_secret`: 原始密钥（解密后的）
    /// - `code`: TOTP代码（6位数字）
    /// 
    /// # 返回
    /// 如果代码有效返回true，否则返回false
    fn validate_totp(&self, raw_secret: &str, code: u32) -> bool;
    
    /// 生成TOTP密钥（Base32编码）
    /// 
    /// # 返回
    /// Base32编码的密钥字符串
    fn generate_totp_secret(&self) -> String;
    
    /// 加密密钥
    /// 
    /// # 参数
    /// - `raw_secret`: 原始密钥
    /// 
    /// # 返回
    /// 加密后的密钥字符串（Hex编码）
    fn encrypt_secret(&self, raw_secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 解密密钥
    /// 
    /// # 参数
    /// - `encrypted_secret`: 加密后的密钥（Hex编码）
    /// 
    /// # 返回
    /// 解密后的原始密钥
    fn decrypt_secret(&self, encrypted_secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认TOTP认证服务实现
pub struct DefaultTotpAuthService {
    cipher: Aes256Gcm,
}

impl DefaultTotpAuthService {
    pub fn new(encryption_key: String) -> Self {
        // 从encryption_key派生AES-256密钥（32字节）
        let mut hasher = Sha256::new();
        hasher.update(encryption_key.as_bytes());
        let key_bytes = hasher.finalize();
        
        // 创建AES-256-GCM密码实例
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .expect("Failed to create AES-GCM cipher");
        
        Self { cipher }
    }
    
    /// AES-GCM加密实现
    fn encrypt_internal(&self, data: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 生成随机nonce（12字节，AES-GCM标准）
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // 加密数据
        let ciphertext = self.cipher.encrypt(&nonce, data.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        // 将nonce和ciphertext组合：nonce(12字节) + ciphertext
        let mut encrypted_data = Vec::with_capacity(12 + ciphertext.len());
        encrypted_data.extend_from_slice(&nonce);
        encrypted_data.extend_from_slice(&ciphertext);
        
        // 使用Hex编码返回
        Ok(hex::encode(encrypted_data))
    }
    
    /// AES-GCM解密实现
    fn decrypt_internal(&self, encrypted: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 从Hex解码
        let encrypted_data = hex::decode(encrypted)
            .map_err(|e| format!("Invalid hex encoding: {}", e))?;
        
        if encrypted_data.len() < 12 {
            return Err("Encrypted data too short".into());
        }
        
        // 提取nonce（前12字节）和ciphertext（剩余部分）
        // Nonce是固定12字节的数组
        let nonce_bytes: [u8; 12] = encrypted_data[..12].try_into()
            .map_err(|_| "Invalid nonce length")?;
        // 使用Nonce::from_slice（虽然已废弃，但这是当前可用的方法）
        // TODO: 升级到generic-array 1.x后使用新API
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = &encrypted_data[12..];
        
        // 解密数据
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        // 转换为字符串
        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8: {}", e).into())
    }
}

#[async_trait]
impl TotpAuthService for DefaultTotpAuthService {
    fn validate_totp(&self, raw_secret: &str, code: u32) -> bool {
        // 使用totp-lite库验证TOTP代码
        // TOTP算法：基于时间的一次性密码
        // 默认参数：SHA1算法，6位数字，30秒时间窗口
        
        // 解码Base32密钥
        let secret_bytes = match base32::decode(base32::Alphabet::RFC4648 { padding: false }, raw_secret) {
            Some(bytes) => bytes,
            None => return false,
        };
        
        // 获取当前时间戳（秒）
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 使用totp-lite生成TOTP代码（允许前后一个时间窗口的容差）
        for offset in -1..=1 {
            let time = timestamp as i64 + offset * 30; // 30秒时间窗口
            if time < 0 {
                continue;
            }
            
            // 使用totp_lite生成代码
            // totp_lite::totp_custom<H>(step, digits, secret, time) -> String
            // H是哈希算法类型，使用Sha1
            use totp_lite::Sha1;
            let generated_code_str = totp_lite::totp_custom::<Sha1>(
                30, // 30秒时间窗口
                6, // 6位数字
                &secret_bytes,
                time as u64,
            );
            
            // 将生成的代码转换为u32进行比较
            if let Ok(gen_code) = generated_code_str.parse::<u32>() {
                if gen_code == code {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn generate_totp_secret(&self) -> String {
        // 生成20字节（160位）的随机密钥，然后Base32编码
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut secret = vec![0u8; 20]; // 160位密钥
        rng.fill(&mut secret[..]);
        
        // Base32编码
        base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret)
    }
    
    fn encrypt_secret(&self, raw_secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.encrypt_internal(raw_secret)
    }
    
    fn decrypt_secret(&self, encrypted_secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.decrypt_internal(encrypted_secret)
    }
}

