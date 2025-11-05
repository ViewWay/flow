#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::extension_store::Model as ExtensionStoreModel;

    #[tokio::test]
    async fn test_extension_store_model_creation() {
        let store = ExtensionStoreModel {
            name: "test-extension".to_string(),
            data: b"test data".to_vec(),
            version: Some(1),
        };

        assert_eq!(store.name, "test-extension");
        assert_eq!(store.data, b"test data");
        assert_eq!(store.version, Some(1));
    }

    #[tokio::test]
    async fn test_extension_store_serialization() {
        use serde_json;

        let store = ExtensionStoreModel {
            name: "test-extension".to_string(),
            data: b"test data".to_vec(),
            version: Some(1),
        };

        let json = serde_json::to_string(&store).unwrap();
        assert!(json.contains("test-extension"));
    }
}

