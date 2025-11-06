use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions, ListResult};
use flow_domain::security::{RoleBinding, Subject};
use std::sync::Arc;

// USER_KIND常量在role_binding模块中定义
const USER_KIND: &str = "User";

/// 角色绑定服务trait
#[async_trait]
pub trait RoleBindingService: Send + Sync {
    /// 创建角色绑定
    async fn create(&self, binding: RoleBinding) -> Result<RoleBinding, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 更新角色绑定
    async fn update(&self, binding: RoleBinding) -> Result<RoleBinding, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 删除角色绑定
    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取角色绑定
    async fn get(&self, name: &str) -> Result<Option<RoleBinding>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 列出角色绑定
    async fn list(&self, options: ListOptions) -> Result<ListResult<RoleBinding>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 根据Subject查找角色绑定
    async fn list_by_subject(&self, subject: &Subject) -> Result<Vec<RoleBinding>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 为用户授予角色
    async fn grant_roles(&self, username: &str, role_names: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// 撤销用户的角色
    async fn revoke_roles(&self, username: &str, role_names: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认角色绑定服务实现
pub struct DefaultRoleBindingService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultRoleBindingService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: ExtensionClient> RoleBindingService for DefaultRoleBindingService<C> {
    async fn create(&self, binding: RoleBinding) -> Result<RoleBinding, Box<dyn std::error::Error + Send + Sync>> {
        self.client.create(binding).await
    }

    async fn update(&self, binding: RoleBinding) -> Result<RoleBinding, Box<dyn std::error::Error + Send + Sync>> {
        self.client.update(binding).await
    }

    async fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.delete::<RoleBinding>(name).await
    }

    async fn get(&self, name: &str) -> Result<Option<RoleBinding>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.fetch(name).await
    }

    async fn list(&self, options: ListOptions) -> Result<ListResult<RoleBinding>, Box<dyn std::error::Error + Send + Sync>> {
        self.client.list(options).await
    }

    async fn list_by_subject(&self, subject: &Subject) -> Result<Vec<RoleBinding>, Box<dyn std::error::Error + Send + Sync>> {
        // 列出所有RoleBinding，然后过滤出包含该subject的
        let options = ListOptions::default();
        let result = self.list(options).await?;
        
        let bindings: Vec<RoleBinding> = result.items
            .into_iter()
            .filter(|binding| {
                binding.subjects.iter().any(|s| {
                    s.kind == subject.kind && s.name == subject.name && s.api_group == subject.api_group
                })
            })
            .collect();
        
        Ok(bindings)
    }

    async fn grant_roles(&self, username: &str, role_names: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 获取用户现有的角色绑定
        let subject = Subject {
            kind: USER_KIND.to_string(),
            name: username.to_string(),
            api_group: "".to_string(),
        };
        
        let existing_bindings = self.list_by_subject(&subject).await?;
        let existing_roles: std::collections::HashSet<String> = existing_bindings
            .iter()
            .map(|b| b.role_ref.name.clone())
            .collect();
        
        // 为每个新角色创建绑定（如果不存在）
        for role_name in role_names {
            if !existing_roles.contains(role_name) {
                let binding = RoleBinding::create(username, role_name);
                self.create(binding).await?;
            }
        }
        
        Ok(())
    }

    async fn revoke_roles(&self, username: &str, role_names: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 获取用户现有的角色绑定
        let subject = Subject {
            kind: USER_KIND.to_string(),
            name: username.to_string(),
            api_group: "".to_string(),
        };
        
        let existing_bindings = self.list_by_subject(&subject).await?;
        
        // 删除匹配的角色绑定
        for binding in existing_bindings {
            if role_names.contains(&binding.role_ref.name) {
                // 如果绑定只包含一个subject，直接删除
                if binding.subjects.len() == 1 {
                    self.delete(&binding.metadata.name).await?;
                } else {
                    // 否则从subjects中移除该用户
                    let mut updated_binding = binding.clone();
                    updated_binding.subjects.retain(|s| s.name != username);
                    if !updated_binding.subjects.is_empty() {
                        self.update(updated_binding).await?;
                    } else {
                        // 如果移除后subjects为空，删除绑定
                        self.delete(&binding.metadata.name).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

