use std::fmt::Debug;
use serde::{Deserialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct FacetFields {
    #[serde(flatten)]
    pub fields: Value,
    #[serde(skip)]
    field: String
}

impl FacetFields {

    pub fn field(&mut self, value: &str) -> &mut Self {
        self.field = value.to_string();
        self
    }

    pub fn get_all(&self) -> Option<Vec<&str>> {
        let mut array_iter = match self.fields[&self.field].as_array() {
            Some(ar) => ar.iter(),
            None => return None
        };
        let mut all = vec![];

        while let Some(i) = array_iter.next()  {
            if i.is_string() {
               all.push(i.as_str().unwrap());
            }
        }
        Some(all)
    }

    pub fn get(&self, key: &str) -> Option<u64> {
        let mut array_iter = match self.fields[&self.field].as_array() {
            Some(ar) => ar.iter(),
            None => return None
        };

        let mut actual = array_iter.next();
        while actual != None  {

            match actual.unwrap().as_str() {
                Some(str) => {
                    if str == key {
                        return array_iter.next().unwrap().as_u64()
                    }
                },
                None => ()
            }

            actual = array_iter.next();
        }
        None
    }

}

#[cfg(test)]
mod tests {
    use crate::facet_fields::FacetFields;

    #[test]
    fn test_get_custom_field_count() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields, field: "".to_string() };

        assert_eq!(facet_fields.field("field_value").get("val2"), Some(234))
    }

    #[test]
    fn test_returns_none_if_no_field() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields, field: "".to_string() };

        assert_eq!(facet_fields.field("not_existing").get("val1"), None);
    }

    #[test]
    fn test_returns_none_if_no_field_value() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields, field: "".to_string() };

        assert_eq!(facet_fields.field("field_value").get("not_existing"), None);
    }

    #[test]
    fn test_get_all_field_values() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields, field: "".to_string() };

        let expected_values = vec!["val1", "val2"];

        assert_eq!(facet_fields.field("field_value").get_all(), Some(expected_values));
    }

    #[test]
    fn test_get_all_none_from_notexisting_field() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields, field: "".to_string() };
        assert_eq!(facet_fields.field("field_value2342").get_all(), None);
    }

}
