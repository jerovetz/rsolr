use std::fmt::Debug;
use serde::{Deserialize};
use serde_json::Value;

/// The fields part of the facets.
#[derive(Clone, Debug, Deserialize)]
pub struct FacetFields {
    #[serde(flatten)]
    pub fields: Value,
}

impl FacetFields {

    pub fn get_all_values_of(&self, field: &str) -> Option<Vec<&str>> {
        let mut array_iter = match self.fields[field].as_array() {
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

    pub fn get_count(&self, field: &str, key: &str) -> Option<u64> {
        let mut array_iter = match self.fields[field].as_array() {
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
        let mut facet_fields = FacetFields { fields };

        assert_eq!(facet_fields.get_count("field_value","val2"), Some(234))
    }

    #[test]
    fn test_returns_none_if_no_field() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields };

        assert_eq!(facet_fields.get_count("not_existing","val1"), None);
    }

    #[test]
    fn test_returns_none_if_no_field_value() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields };

        assert_eq!(facet_fields.get_count("field_value", "not_existing"), None);
    }

    #[test]
    fn test_get_all_field_values() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields };

        let expected_values = vec!["val1", "val2"];

        assert_eq!(facet_fields.get_all_values_of("field_value"), Some(expected_values));
    }

    #[test]
    fn test_get_all_none_from_notexisting_field() {
        let fields = serde_json::from_str(r#"{"field_value": ["val1", 123, "val2", 234] }"#).unwrap();
        let mut facet_fields = FacetFields { fields };
        assert_eq!(facet_fields.get_all_values_of("field_value2342"), None);
    }

}
