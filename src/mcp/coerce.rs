use rmcp::model::JsonObject;
use serde_json::Value;

/// Some MCP clients (Google ADK, and kagent on top of it) send every argument
/// as a string, so arrays and objects arrive JSON-encoded or comma-separated.
pub fn coerce_arguments(schema: &JsonObject, arguments: &mut JsonObject) {
    let Some(Value::Object(properties)) = schema.get("properties") else {
        return;
    };
    for (key, value) in arguments.iter_mut() {
        let Some(property) = properties.get(key) else {
            continue;
        };
        let Value::String(text) = value else {
            continue;
        };
        let text = text.trim();
        let parsed = serde_json::from_str::<Value>(text).ok();
        if declares(property, "array") {
            *value = parsed.filter(Value::is_array).unwrap_or_else(|| {
                text.split(',')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(Value::from)
                    .collect()
            });
        } else if declares(property, "object")
            && let Some(object) = parsed.filter(Value::is_object)
        {
            *value = object;
        }
    }
}

fn declares(schema: &Value, kind: &str) -> bool {
    match schema.get("type") {
        Some(Value::String(t)) => t == kind,
        Some(Value::Array(types)) => types.iter().any(|t| t == kind),
        _ => false,
    }
}

pub fn strip_defaults(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("default");
            map.values_mut().for_each(strip_defaults);
        }
        Value::Array(items) => items.iter_mut().for_each(strip_defaults),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn object(value: Value) -> JsonObject {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn coerces_stringified_arrays_and_objects() {
        let schema = object(json!({"properties": {
            "ids": {"type": "array", "items": {"type": "string"}},
            "filter": {"type": ["object", "null"]},
            "start": {"type": "string"}
        }}));
        let mut arguments = object(json!({
            "ids": "[\"123456789012\"]",
            "filter": "{\"Region\":[\"APN2\"]}",
            "start": "2026-09-01",
            "unknown": "[1]"
        }));
        coerce_arguments(&schema, &mut arguments);
        assert_eq!(
            Value::Object(arguments),
            json!({
                "ids": ["123456789012"],
                "filter": {"Region": ["APN2"]},
                "start": "2026-09-01",
                "unknown": "[1]"
            })
        );
    }

    #[test]
    fn splits_plain_strings_for_array_fields() {
        let schema = object(json!({"properties": {"ids": {"type": "array"}}}));
        let mut arguments = object(json!({"ids": "123456789012, 210987654321"}));
        coerce_arguments(&schema, &mut arguments);
        assert_eq!(arguments["ids"], json!(["123456789012", "210987654321"]));

        let mut untouched = object(json!({"ids": ["a"]}));
        coerce_arguments(&schema, &mut untouched);
        assert_eq!(untouched["ids"], json!(["a"]));

        let mut no_schema = object(json!({"ids": "a"}));
        coerce_arguments(&JsonObject::new(), &mut no_schema);
        assert_eq!(no_schema["ids"], json!("a"));
    }

    #[test]
    fn strips_default_annotations() {
        let mut schema = json!({"default": 1, "properties": {"a": {"default": "", "anyOf": [{"default": null}]}}});
        strip_defaults(&mut schema);
        assert_eq!(schema, json!({"properties": {"a": {"anyOf": [{}]}}}));
    }
}
