/// Patch工具，用于合并Snapshot
/// 实现类似Halo的diff/patch逻辑
use serde::{Deserialize, Serialize};

/// Delta类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeltaType {
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "INSERT")]
    Insert,
    #[serde(rename = "CHANGE")]
    Change,
}

/// Delta表示一个变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub source: StringChunk,
    pub target: StringChunk,
    #[serde(rename = "type")]
    pub delta_type: DeltaType,
}

/// StringChunk表示一个文本块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringChunk {
    pub position: i32,
    pub lines: Vec<String>,
    #[serde(rename = "changePosition")]
    pub change_position: Vec<i32>,
}

/// 应用patch到原始内容
pub fn apply_patch(original: &str, patch_json: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 解析patch JSON
    let deltas: Vec<Delta> = serde_json::from_str(patch_json)?;
    
    // 将原始内容按行分割
    let mut lines: Vec<String> = if original.is_empty() {
        Vec::new()
    } else {
        original.lines().map(|s| s.to_string()).collect()
    };
    
    // 按位置倒序排序deltas，从后往前应用，避免位置偏移问题
    let mut sorted_deltas = deltas;
    sorted_deltas.sort_by(|a, b| b.source.position.cmp(&a.source.position));
    
    // 应用每个delta
    for delta in sorted_deltas {
        match delta.delta_type {
            DeltaType::Delete => {
                // 删除操作：删除source位置的lines
                let start = delta.source.position as usize;
                let end = (start + delta.source.lines.len()).min(lines.len());
                if start < lines.len() {
                    lines.drain(start..end);
                }
            }
            DeltaType::Insert => {
                // 插入操作：在source位置插入target的lines
                let pos = delta.source.position as usize;
                if pos <= lines.len() {
                    let insert_lines = delta.target.lines.clone();
                    for (i, line) in insert_lines.iter().enumerate() {
                        lines.insert(pos + i, line.clone());
                    }
                }
            }
            DeltaType::Change => {
                // 修改操作：替换source位置的lines为target的lines
                let start = delta.source.position as usize;
                let end = (start + delta.source.lines.len()).min(lines.len());
                if start < lines.len() {
                    lines.drain(start..end);
                    let replace_lines = delta.target.lines.clone();
                    for (i, line) in replace_lines.iter().enumerate() {
                        lines.insert(start + i, line.clone());
                    }
                }
            }
        }
    }
    
    // 重新组合为字符串
    Ok(lines.join("\n"))
}

/// 生成diff patch（简化版本，实际应该使用更复杂的diff算法）
/// 这里提供一个基础实现，实际生产环境应该使用更成熟的diff库
pub fn diff_to_json_patch(original: &str, revised: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 简化实现：如果内容相同，返回空patch
    if original == revised {
        return Ok("[]".to_string());
    }
    
    // 简单的逐行diff实现
    let original_lines: Vec<String> = if original.is_empty() {
        Vec::new()
    } else {
        original.lines().map(|s| s.to_string()).collect()
    };
    
    let revised_lines: Vec<String> = if revised.is_empty() {
        Vec::new()
    } else {
        revised.lines().map(|s| s.to_string()).collect()
    };
    
    // 使用简单的LCS（最长公共子序列）算法
    let mut deltas = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < original_lines.len() || j < revised_lines.len() {
        if i < original_lines.len() && j < revised_lines.len() && original_lines[i] == revised_lines[j] {
            // 行匹配，继续
            i += 1;
            j += 1;
        } else if j < revised_lines.len() && (i >= original_lines.len() || (i < original_lines.len() && j < revised_lines.len() && original_lines[i] != revised_lines[j])) {
            // 需要插入
            let mut insert_lines = Vec::new();
            let start_j = j;
            while j < revised_lines.len() && (i >= original_lines.len() || original_lines[i] != revised_lines[j]) {
                insert_lines.push(revised_lines[j].clone());
                j += 1;
            }
            deltas.push(Delta {
                source: StringChunk {
                    position: i as i32,
                    lines: Vec::new(),
                    change_position: Vec::new(),
                },
                target: StringChunk {
                    position: i as i32,
                    lines: insert_lines,
                    change_position: (start_j..j).map(|x| x as i32).collect(),
                },
                delta_type: DeltaType::Insert,
            });
        } else if i < original_lines.len() {
            // 需要删除
            let mut delete_lines = Vec::new();
            let start_i = i;
            while i < original_lines.len() && (j >= revised_lines.len() || original_lines[i] != revised_lines[j]) {
                delete_lines.push(original_lines[i].clone());
                i += 1;
            }
            deltas.push(Delta {
                source: StringChunk {
                    position: start_i as i32,
                    lines: delete_lines,
                    change_position: Vec::new(),
                },
                target: StringChunk {
                    position: start_i as i32,
                    lines: Vec::new(),
                    change_position: Vec::new(),
                },
                delta_type: DeltaType::Delete,
            });
        }
    }
    
    serde_json::to_string(&deltas).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_patch_simple() {
        let original = "line1\nline2\nline3";
        let patch = r#"[{"type":"CHANGE","source":{"position":1,"lines":["line2"],"changePosition":[]},"target":{"position":1,"lines":["line2_modified"],"changePosition":[]}}]"#;
        let result = apply_patch(original, patch).unwrap();
        assert_eq!(result, "line1\nline2_modified\nline3");
    }
}

