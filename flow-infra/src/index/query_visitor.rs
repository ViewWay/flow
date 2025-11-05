use flow_api::extension::Extension;
use flow_api::extension::query::Condition;
use flow_api::extension::index::LabelIndexQuery;
use crate::index::indices::Indices;
use std::collections::HashSet;
use std::sync::Arc;

/// QueryVisitor 遍历查询条件并执行查询
pub struct QueryVisitor<E: Extension + 'static> {
    result: HashSet<String>,
    indices: Arc<Indices<E>>,
}

impl<E: Extension + 'static> QueryVisitor<E> {
    pub fn new(indices: Arc<Indices<E>>) -> Self {
        Self {
            result: HashSet::new(),
            indices,
        }
    }
    
    pub fn visit(&mut self, condition: &Condition) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match condition {
            Condition::Empty => {
                // 空条件匹配所有
                // 使用标签索引获取所有主键
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                self.result.extend(all_keys);
            }
            
            Condition::And { left, right } => {
                self.visit(left)?;
                if !self.result.is_empty() {
                    let mut right_result = HashSet::new();
                    Self::visit_condition(right, &self.indices, &mut right_result)?;
                    self.result = self.result.intersection(&right_result).cloned().collect();
                }
            }
            
            Condition::Or { left, right } => {
                self.visit(left)?;
                let mut right_result = HashSet::new();
                Self::visit_condition(right, &self.indices, &mut right_result)?;
                self.result.extend(right_result);
            }
            
            Condition::Not { condition } => {
                // 获取所有主键，然后移除匹配的
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                
                let mut matched = HashSet::new();
                Self::visit_condition(condition, &self.indices, &mut matched)?;
                
                self.result = all_keys.difference(&matched).cloned().collect();
            }
            
            Condition::Equal { index_name, value } => {
                if index_name == "metadata.labels" {
                    // 这是标签查询，不应该在这里
                    return Err("Label queries should use LabelEquals condition".into());
                }
                let result = self.indices.query_equal(index_name, value)?;
                self.result.extend(result);
            }
            
            Condition::NotEqual { index_name, value } => {
                if index_name == "metadata.labels" {
                    return Err("Label queries should use LabelNotEquals condition".into());
                }
                let result = self.indices.query_not_equal(index_name, value)?;
                self.result.extend(result);
            }
            
            Condition::In { index_name, values } => {
                if index_name == "metadata.labels" {
                    return Err("Label queries should use LabelIn condition".into());
                }
                let result = self.indices.query_in(index_name, values)?;
                self.result.extend(result);
            }
            
            Condition::NotIn { index_name, values } => {
                if index_name == "metadata.labels" {
                    return Err("Label queries should use LabelNotIn condition".into());
                }
                let result = self.indices.query_not_in(index_name, values)?;
                self.result.extend(result);
            }
            
            Condition::LessThan { index_name, bound, inclusive } => {
                let result = self.indices.query_less_than(index_name, bound, *inclusive)?;
                self.result.extend(result);
            }
            
            Condition::GreaterThan { index_name, bound, inclusive } => {
                let result = self.indices.query_greater_than(index_name, bound, *inclusive)?;
                self.result.extend(result);
            }
            
            Condition::Between { index_name, from_key, from_inclusive, to_key, to_inclusive } => {
                let result = self.indices.query_between(
                    index_name,
                    from_key,
                    *from_inclusive,
                    to_key,
                    *to_inclusive,
                )?;
                self.result.extend(result);
            }
            
            Condition::NotBetween { index_name, from_key, from_inclusive, to_key, to_inclusive } => {
                let result = self.indices.query_not_between(
                    index_name,
                    from_key,
                    *from_inclusive,
                    to_key,
                    *to_inclusive,
                )?;
                self.result.extend(result);
            }
            
            Condition::IsNull { index_name } => {
                // IS NULL查询：获取所有主键，然后移除索引中存在的
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                let indexed_keys = self.indices.query_all(index_name)
                    .unwrap_or_default();
                self.result = all_keys.difference(&indexed_keys).cloned().collect();
            }
            
            Condition::IsNotNull { index_name } => {
                // IS NOT NULL查询：直接获取索引中的所有主键
                let result = self.indices.query_all(index_name)?;
                self.result.extend(result);
            }
            
            Condition::LabelExists { label_key } => {
                let label_index = self.indices.label_index();
                let result = label_index.exists(label_key);
                self.result.extend(result);
            }
            
            Condition::LabelNotExists { label_key } => {
                // 获取所有主键，然后移除有该标签的
                let label_index = self.indices.label_index();
                let all_keys = label_index.all_primary_keys();
                let exists_result = label_index.exists(label_key);
                self.result = all_keys.difference(&exists_result).cloned().collect();
            }
            
            Condition::LabelEquals { label_key, label_value } => {
                let label_index = self.indices.label_index();
                let result = label_index.equal(label_key, label_value);
                self.result.extend(result);
            }
            
            Condition::LabelNotEquals { label_key, label_value } => {
                let label_index = self.indices.label_index();
                let result = label_index.not_equal(label_key, label_value);
                self.result.extend(result);
            }
            
            Condition::LabelIn { label_key, label_values } => {
                let label_index = self.indices.label_index();
                let result = label_index.in_set(label_key, label_values);
                self.result.extend(result);
            }
            
            Condition::LabelNotIn { label_key, label_values } => {
                let label_index = self.indices.label_index();
                let result = label_index.not_in_set(label_key, label_values);
                self.result.extend(result);
            }
        }
        
        Ok(())
    }
    
    fn visit_condition(
        condition: &Condition,
        indices: &Arc<Indices<E>>,
        result: &mut HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut visitor = Self::new(Arc::clone(indices));
        visitor.visit(condition)?;
        result.extend(visitor.result);
        Ok(())
    }
    
    pub fn get_result(self) -> HashSet<String> {
        self.result
    }
}

