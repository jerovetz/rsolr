use std::fmt::Debug;
use serde::{Deserialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct FacetFields {
    #[serde(flatten)]
    pub fields: Value,
    field_value: String
}

impl FacetFields {

    pub fn field(&mut self, value: String) -> &mut Self {
        self.field_value = value;
        self
    }

    pub fn get(&self, key: String) -> u64 {
        let mut array_iter = self.fields[&self.field_value].as_array().unwrap().iter();
        while array_iter.next().unwrap().as_str().unwrap() != key { }

        array_iter.next().unwrap().as_u64().unwrap()
    }

}

#[cfg(test)]
mod tests {
    use crate::facet_fields::FacetFields;

    #[test]
    fn test_get_custom_field_count() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields, field_value: "".to_string() };

        assert_eq!(facet_fields.field("field_value".to_string()).get("val1".to_string()), 123)
    }

}
