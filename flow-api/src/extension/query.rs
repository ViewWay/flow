use serde::{Deserialize, Serialize};

/// Condition 表示查询条件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    /// 空条件（匹配所有）
    Empty,
    
    /// AND条件
    And {
        left: Box<Condition>,
        right: Box<Condition>,
    },
    
    /// OR条件
    Or {
        left: Box<Condition>,
        right: Box<Condition>,
    },
    
    /// NOT条件
    Not {
        condition: Box<Condition>,
    },
    
    /// 等于条件
    Equal {
        index_name: String,
        value: serde_json::Value,
    },
    
    /// 不等于条件
    NotEqual {
        index_name: String,
        value: serde_json::Value,
    },
    
    /// IN条件
    In {
        index_name: String,
        values: Vec<serde_json::Value>,
    },
    
    /// NOT IN条件
    NotIn {
        index_name: String,
        values: Vec<serde_json::Value>,
    },
    
    /// 小于条件
    LessThan {
        index_name: String,
        bound: serde_json::Value,
        inclusive: bool,
    },
    
    /// 大于条件
    GreaterThan {
        index_name: String,
        bound: serde_json::Value,
        inclusive: bool,
    },
    
    /// BETWEEN条件
    Between {
        index_name: String,
        from_key: serde_json::Value,
        from_inclusive: bool,
        to_key: serde_json::Value,
        to_inclusive: bool,
    },
    
    /// NOT BETWEEN条件
    NotBetween {
        index_name: String,
        from_key: serde_json::Value,
        from_inclusive: bool,
        to_key: serde_json::Value,
        to_inclusive: bool,
    },
    
    /// IS NULL条件
    IsNull {
        index_name: String,
    },
    
    /// IS NOT NULL条件
    IsNotNull {
        index_name: String,
    },
    
    /// 标签存在条件
    LabelExists {
        label_key: String,
    },
    
    /// 标签不存在条件
    LabelNotExists {
        label_key: String,
    },
    
    /// 标签等于条件
    LabelEquals {
        label_key: String,
        label_value: String,
    },
    
    /// 标签不等于条件
    LabelNotEquals {
        label_key: String,
        label_value: String,
    },
    
    /// 标签IN条件
    LabelIn {
        label_key: String,
        label_values: Vec<String>,
    },
    
    /// 标签NOT IN条件
    LabelNotIn {
        label_key: String,
        label_values: Vec<String>,
    },
}

impl Condition {
    /// 创建空条件
    pub fn empty() -> Self {
        Self::Empty
    }
    
    /// AND组合
    pub fn and(self, other: Condition) -> Self {
        Self::And {
            left: Box::new(self),
            right: Box::new(other),
        }
    }
    
    /// OR组合
    pub fn or(self, other: Condition) -> Self {
        Self::Or {
            left: Box::new(self),
            right: Box::new(other),
        }
    }
    
    /// NOT取反
    pub fn not(self) -> Self {
        Self::Not {
            condition: Box::new(self),
        }
    }
}

/// Queries 提供查询构建工具函数
pub mod queries {
    use super::Condition;
    use serde_json::Value;
    
    /// 创建等于条件
    pub fn equal(index_name: impl Into<String>, value: Value) -> Condition {
        Condition::Equal {
            index_name: index_name.into(),
            value,
        }
    }
    
    /// 创建不等于条件
    pub fn not_equal(index_name: impl Into<String>, value: Value) -> Condition {
        Condition::NotEqual {
            index_name: index_name.into(),
            value,
        }
    }
    
    /// 创建IN条件
    pub fn in_condition(index_name: impl Into<String>, values: Vec<Value>) -> Condition {
        if values.len() == 1 {
            equal(index_name, values.into_iter().next().unwrap())
        } else {
            Condition::In {
                index_name: index_name.into(),
                values,
            }
        }
    }
    
    /// 创建NOT IN条件
    pub fn not_in(index_name: impl Into<String>, values: Vec<Value>) -> Condition {
        Condition::NotIn {
            index_name: index_name.into(),
            values,
        }
    }
    
    /// 创建小于条件
    pub fn less_than(index_name: impl Into<String>, bound: Value, inclusive: bool) -> Condition {
        Condition::LessThan {
            index_name: index_name.into(),
            bound,
            inclusive,
        }
    }
    
    /// 创建大于条件
    pub fn greater_than(index_name: impl Into<String>, bound: Value, inclusive: bool) -> Condition {
        Condition::GreaterThan {
            index_name: index_name.into(),
            bound,
            inclusive,
        }
    }
    
    /// 创建BETWEEN条件
    pub fn between(
        index_name: impl Into<String>,
        from_key: Value,
        from_inclusive: bool,
        to_key: Value,
        to_inclusive: bool,
    ) -> Condition {
        Condition::Between {
            index_name: index_name.into(),
            from_key,
            from_inclusive,
            to_key,
            to_inclusive,
        }
    }
    
    /// 创建IS NULL条件
    pub fn is_null(index_name: impl Into<String>) -> Condition {
        Condition::IsNull {
            index_name: index_name.into(),
        }
    }
    
    /// 创建IS NOT NULL条件
    pub fn is_not_null(index_name: impl Into<String>) -> Condition {
        Condition::IsNotNull {
            index_name: index_name.into(),
        }
    }
    
    /// 创建标签存在条件
    pub fn label_exists(label_key: impl Into<String>) -> Condition {
        Condition::LabelExists {
            label_key: label_key.into(),
        }
    }
    
    /// 创建标签等于条件
    pub fn label_equal(label_key: impl Into<String>, label_value: impl Into<String>) -> Condition {
        Condition::LabelEquals {
            label_key: label_key.into(),
            label_value: label_value.into(),
        }
    }
    
    /// 创建标签IN条件
    pub fn label_in(label_key: impl Into<String>, label_values: Vec<String>) -> Condition {
        Condition::LabelIn {
            label_key: label_key.into(),
            label_values,
        }
    }
}

