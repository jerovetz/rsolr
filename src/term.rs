pub struct Term {
    term: String,
    field: Option<String>
}

impl Term {

    pub fn from_str(term_str: &str) -> Self {
        if term_str.find(" ") != None {
            return Term{ term: format!("\"{}\"", term_str.to_owned()), field: None };
        }
        Term{ term: term_str.to_owned(), field: None }
    }

    pub fn in_field(&mut self, field: &str) -> &mut Self {
        self.field = Some(field.to_owned());
        self
    }

    pub fn boost(&mut self, value: f32) -> &mut Self {
        self.term = format!("{}^{}", self.term, value.to_string());
        self
    }

    pub fn tilde(&mut self, value: u32) -> &mut Self {
        self.term = format!("{}~{}", self.term, value.to_string());
        self
    }

    pub fn required(&mut self) -> &mut Self {
        self.term = format!("+{}", self.term);
        self
    }

    pub fn prohibit(&mut self) -> &mut Self {
        self.term = format!("-{}", self.term);
        self
    }

    pub fn as_str(&self) -> String {
        match &self.field {
            Some(field) => format!("{}: {}", field, self.term),
            None => self.term.clone()
        }
    }


}

mod tests {
    use crate::term::Term;

    #[test]
    fn test_as_str_returns_term_as_str_in_quotes() {
        let term = "term term";
        assert_eq!(Term::from_str(term).as_str(), format!("\"{}\"", term));
    }

    #[test]
    fn test_as_str_returns_term_without_quotes() {
        let term = "term";
        assert_eq!(Term::from_str(term).as_str(), term);
    }

    #[test]
    fn test_in_field_decorate_it_with_field() {
        let term_str = "term term";
        let mut term = Term::from_str(term_str);
        term.in_field("field");
        assert_eq!(term.as_str(), "field: \"term term\"");
    }

    #[test]
    fn test_boost_term_chained_with_field() {
        let mut term = Term::from_str("term term");
        let term_str = term.in_field("field").boost(3.2).as_str();
        assert_eq!(term_str, "field: \"term term\"^3.2");
    }

    #[test]
    fn test_tile_term_chained_with_boost() {
        let mut term = Term::from_str("term term");
        let term_str = term.boost(3.2).tilde(20).as_str();
        assert_eq!(term_str, "\"term term\"^3.2~20");
    }

    #[test]
    fn test_require_term() {
        let mut term = Term::from_str("term");
        let term_str = term.required().as_str();
        assert_eq!(term_str, "+term");
    }

    #[test]
    fn test_prohibit_term() {
        let mut term = Term::from_str("term");
        let term_str = term.prohibit().as_str();
        assert_eq!(term_str, "-term");
    }

}