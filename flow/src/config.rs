use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub mongodb: MongoDBConfig,
    pub flow: FlowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: Option<usize>,
    pub max_request_body_size: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8090,
            host: "0.0.0.0".to_string(),
            workers: None,
            max_request_body_size: "10MB".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub mysql: Option<DatabaseConnectionConfig>,
    pub postgresql: Option<DatabaseConnectionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnectionConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBConfig {
    pub url: String,
    pub database: String,
}

impl Default for MongoDBConfig {
    fn default() -> Self {
        Self {
            url: "mongodb://localhost:27017".to_string(),
            database: "flow".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    pub work_dir: PathBuf,
    pub external_url: Option<String>,
    pub use_absolute_permalink: bool,
    pub security: SecurityConfig,
    pub cache: CacheConfig,
    pub search: SearchConfig,
    pub plugin: PluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub bcrypt_cost: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "change-me-in-production".to_string(),
            jwt_expiration: 3600,
            bcrypt_cost: 12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(rename = "type")]
    pub cache_type: String,
    pub memory_max_size: usize,
    pub memory_ttl: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: "redis".to_string(),
            memory_max_size: 10000,
            memory_ttl: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub engine: String,
    pub index_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub runtime: String,
    pub plugins_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let work_dir = home_dir.join(".flow2");

        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig {
                mysql: None,
                postgresql: None,
            },
            redis: RedisConfig::default(),
            mongodb: MongoDBConfig::default(),
            flow: FlowConfig {
                work_dir: work_dir.clone(),
                external_url: None,
                use_absolute_permalink: false,
                security: SecurityConfig::default(),
                cache: CacheConfig::default(),
                search: SearchConfig {
                    engine: "tantivy".to_string(),
                    index_path: work_dir.join("indices"),
                },
                plugin: PluginConfig {
                    runtime: "ffi".to_string(),
                    plugins_dir: work_dir.join("plugins"),
                },
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let work_dir = home_dir.join(".flow2");
        let config_path = work_dir.join("flow.toml");

        let mut builder = config::Config::builder()
            .add_source(config::File::with_name("flow.toml").required(false))
            .add_source(
                config::File::from(config_path.as_path())
                    .required(false),
            )
            .add_source(config::Environment::with_prefix("FLOW").separator("__"));

        // 如果存在.env文件，加载它
        if let Ok(_) = dotenv::dotenv() {
            builder = builder.add_source(config::Environment::with_prefix("FLOW").separator("__"));
        }

        let config = builder.build()?;
        config.try_deserialize()
    }
}

