use async_trait::async_trait;
use flow_api::extension::{ExtensionClient, ListOptions};
use flow_domain::security::{Role, RoleBinding};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// 角色服务trait
#[async_trait]
pub trait RoleService: Send + Sync {
    /// 列出角色依赖（包括角色本身和所有依赖的角色）
    async fn list_dependencies(&self, role_names: &[String]) 
        -> Result<Vec<Role>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 解析聚合角色（通过标签匹配）
    async fn resolve_aggregated_roles(&self, role_name: &str) 
        -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// 获取用户的所有角色
    async fn get_user_roles(&self, username: &str) 
        -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认角色服务实现
/// 注意：由于ExtensionClient trait的限制，这里使用泛型而不是trait object
pub struct DefaultRoleService<C: ExtensionClient> {
    client: Arc<C>,
}

impl<C: ExtensionClient> DefaultRoleService<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// 构建角色依赖图
    async fn build_dependency_graph(&self) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut graph = HashMap::new();
        
        // 列出所有角色
        let options = ListOptions::default();
        let roles: Vec<Role> = self.client.list(options).await?.items;
        
        for role in roles {
            let role_name = role.metadata.name.clone();
            let mut dependencies = Vec::new();
            
            // 从annotations中读取依赖关系
            if let Some(annotations) = role.metadata.annotations.as_ref() {
                if let Some(deps_json) = annotations.get("rbac.authorization.halo.run/dependencies") {
                    // 解析JSON格式的依赖列表
                    if let Ok(deps) = serde_json::from_str::<Vec<String>>(deps_json) {
                        dependencies = deps;
                    }
                }
            }
            
            graph.insert(role_name, dependencies);
        }
        
        Ok(graph)
    }

    /// 深度优先搜索获取所有依赖角色（同步版本，因为graph已经构建完成）
    fn get_all_dependencies(
        &self,
        role_name: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(role_name) {
            return;
        }
        
        visited.insert(role_name.to_string());
        result.push(role_name.to_string());
        
        // 递归获取依赖角色
        if let Some(dependencies) = graph.get(role_name) {
            for dep in dependencies {
                self.get_all_dependencies(dep, graph, visited, result);
            }
        }
    }
}

#[async_trait]
impl<C: ExtensionClient> RoleService for DefaultRoleService<C> {
    async fn list_dependencies(&self, role_names: &[String]) 
        -> Result<Vec<Role>, Box<dyn std::error::Error + Send + Sync>> {
        let graph = self.build_dependency_graph().await?;
        let mut visited = HashSet::new();
        let mut role_name_list = Vec::new();
        
        // 获取所有依赖的角色名称
        for role_name in role_names {
            self.get_all_dependencies(role_name, &graph, &mut visited, &mut role_name_list);
        }
        
        // 获取所有角色对象
        let mut roles = Vec::new();
        for role_name in role_name_list {
            if let Ok(Some(role)) = self.client.fetch::<Role>(&role_name).await {
                roles.push(role);
            }
        }
        
        Ok(roles)
    }

    async fn resolve_aggregated_roles(&self, role_name: &str) 
        -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // 查找标签匹配 "rbac.authorization.halo.run/aggregate-to-{role_name}" 的角色
        let label_selector = format!("rbac.authorization.halo.run/aggregate-to-{}={}", role_name, "true");
        let options = ListOptions {
            label_selector: Some(label_selector),
            ..Default::default()
        };
        
        let roles: Vec<Role> = self.client.list(options).await?.items;
        Ok(roles.into_iter().map(|r| r.metadata.name.clone()).collect())
    }

    async fn get_user_roles(&self, username: &str) 
        -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // 查找所有包含该用户的RoleBinding
        let options = ListOptions::default();
        let bindings: Vec<RoleBinding> = self.client.list(options).await?.items;
        
        let mut roles = Vec::new();
        for binding in bindings {
            if binding.contains_user(username) {
                roles.push(binding.role_ref.name.clone());
            }
        }
        
        Ok(roles)
    }
}

