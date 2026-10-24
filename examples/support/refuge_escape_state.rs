//! Canonical, treatment-normalized complete pre-world-state witness.
use serde_json::Value;

/// Normalize only the focal generalized table and update flag. The caller
/// retains exact comparison bytes rather than a large parsed state tree.
pub fn normalized_pre_world_state_bytes(mut state: Value, focal_id: usize) -> Vec<u8> {
    if let Some(focal) = state["entities"]
        .as_array_mut()
        .and_then(|entities| entities.iter_mut().find(|entity| entity["id"] == focal_id))
    {
        focal["memory"]["refuge_generalization_update_enabled"] = Value::Bool(true);
        focal["memory"]["refuge_generalized_outcomes"] = Value::Array(Vec::new());
    }
    serde_json::to_vec(&state).expect("serialize normalized full pre-world state")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const FOCAL_ID: usize = 15;

    #[test]
    fn canonical_state_bytes_normalize_only_focal_table_and_update_flag() {
        let first = json!({
            "rng": 41,
            "map": [[1, 2], [3, 4]],
            "entities": [
                {"id": FOCAL_ID, "pos": {"x": 4, "y": 8}, "memory": {
                    "refuge_generalization_enabled": true,
                    "refuge_generalization_update_enabled": true,
                    "refuge_generalized_outcomes": []
                }},
                {"id": 29, "pos": {"x": 9, "y": 2}, "memory": {"sites": []}}
            ]
        });
        let mut treatment_variant = first.clone();
        treatment_variant["entities"][0]["memory"]["refuge_generalization_update_enabled"] =
            Value::Bool(false);
        treatment_variant["entities"][0]["memory"]["refuge_generalized_outcomes"] =
            json!([{"completed": 3, "threat_damage": 7}]);
        assert_eq!(
            normalized_pre_world_state_bytes(first.clone(), FOCAL_ID),
            normalized_pre_world_state_bytes(treatment_variant.clone(), FOCAL_ID)
        );

        treatment_variant["rng"] = json!(42);
        assert_ne!(
            normalized_pre_world_state_bytes(first.clone(), FOCAL_ID),
            normalized_pre_world_state_bytes(treatment_variant.clone(), FOCAL_ID)
        );
        treatment_variant = first.clone();
        treatment_variant["entities"][0]["memory"]["refuge_generalization_enabled"] =
            Value::Bool(false);
        assert_ne!(
            normalized_pre_world_state_bytes(first.clone(), FOCAL_ID),
            normalized_pre_world_state_bytes(treatment_variant, FOCAL_ID)
        );
        treatment_variant = first.clone();
        treatment_variant["entities"][1]["pos"]["x"] = json!(10);
        assert_ne!(
            normalized_pre_world_state_bytes(first, FOCAL_ID),
            normalized_pre_world_state_bytes(treatment_variant, FOCAL_ID)
        );
    }

    #[test]
    fn comparison_witness_storage_is_canonical_bytes_not_state_tree() {
        let state = json!({
            "rng": 41,
            "entities": [{"id": FOCAL_ID, "memory": {
                "refuge_generalization_update_enabled": false,
                "refuge_generalized_outcomes": [{"large_old_table": "x".repeat(4096)}]
            }}]
        });
        let serialized = normalized_pre_world_state_bytes(state.clone(), FOCAL_ID);
        let raw_size = serde_json::to_vec(&state).unwrap().len();
        assert!(serialized.len() < raw_size);
        assert_eq!(
            serialized,
            serde_json::to_vec(&serde_json::from_slice::<Value>(&serialized).unwrap()).unwrap()
        );
    }
}
