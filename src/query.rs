pub struct Query {
    parts: Vec<Box<dyn Stringable>>
}

pub struct Term {
    term: String,
    field: Option<String>
}

pub trait Stringable {
    fn as_str(&self) -> String;
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
}

impl Stringable for Term {
    fn as_str(&self) -> String {
        match &self.field {
            Some(field) => format!("{}: {}", field, self.term),
            None => self.term.clone()
        }
    }
}

pub struct Date {
    date: String
}

impl Date  {

    pub fn year(count: u32) -> String {
        count.to_string() + "YEARS"
    }

    pub fn month(count: u32) -> String {
        count.to_string() + "MONTHS"
    }

    pub fn day(count: u32) -> String {
        count.to_string() + "DAYS"
    }

    pub fn hour(count: u32) -> String {
        count.to_string() + "HOURS"
    }

    pub fn minute(count: u32) -> String {
        count.to_string() + "MINUTES"
    }

    pub fn second(count: u32) -> String {
        count.to_string() + "SECONDS"
    }

    pub fn new(date_string: &str) -> Self {
        Date { date: date_string.to_owned() }
    }

    pub fn plus(&mut self, duration: &str) -> &mut Self {
        self.date = format!("{}+{}", self.date, duration);
        self
    }

    pub fn minus(&mut self, duration: &str) -> &mut Self {
        self.date = format!("{}-{}", self.date, duration);
        self
    }
}

impl Stringable for Date {
    fn as_str(&self) -> String {
        self.date.clone()
    }
}

pub struct Range<'a> {
    from: &'a str,
    to: &'a str,
    mode: Type
}

#[derive(PartialEq)]
enum Type {
    Inclusive,
    Exclusive
}

impl<'a> Range<'a> {

    pub fn inclusive(from: &'a str, to: &'a str) -> Self {
        Range { from, to, mode: Type::Inclusive }
    }

    pub fn exclusive(from: &'a str, to: &'a str) -> Self {
        Range { from, to, mode: Type::Exclusive }
    }
}

impl<'a> Stringable for Range<'a> {
    fn as_str(&self) -> String {
        if self.mode == Type::Inclusive {
            return format!("[{} TO {}]",self.from, self.to);
        }

        format!("{{{} TO {}}}",self.from, self.to)
    }
}

mod tests {
    use crate::query::{Date, Range, Term, Stringable};

    #[test]
    fn test_term_as_str_returns_term_as_str_in_quotes() {
        let term = "term term";
        assert_eq!(Term::from_str(term).as_str(), format!("\"{}\"", term));
    }

    #[test]
    fn test_term_as_str_returns_term_without_quotes() {
        let term = "term";
        assert_eq!(Term::from_str(term).as_str(), term);
    }

    #[test]
    fn test_term_in_field_decorate_it_with_field() {
        let term_str = "term term";
        let mut term = Term::from_str(term_str);
        term.in_field("field");
        assert_eq!(term.as_str(), "field: \"term term\"");
    }

    #[test]
    fn test_term_boost_term_chained_with_field() {
        let mut term = Term::from_str("term term");
        let term_str = term.in_field("field").boost(3.2).as_str();
        assert_eq!(term_str, "field: \"term term\"^3.2");
    }

    #[test]
    fn test_term_tilde_term_chained_with_boost() {
        let mut term = Term::from_str("term term");
        let term_str = term.boost(3.2).tilde(20).as_str();
        assert_eq!(term_str, "\"term term\"^3.2~20");
    }

    #[test]
    fn test_term_require_term() {
        let mut term = Term::from_str("term");
        let term_str = term.required().as_str();
        assert_eq!(term_str, "+term");
    }

    #[test]
    fn test_term_prohibit_term() {
        let mut term = Term::from_str("term");
        let term_str = term.prohibit().as_str();
        assert_eq!(term_str, "-term");
    }

    #[test]
    fn test_date_as_str_returns_date() {
        let date_string = "NOW";
        assert_eq!(Date::new(date_string).as_str(), date_string);
    }

    #[test]
    fn test_date_plus_concat_text() {
        let date_string = "NOW";
        let expected = "NOW+2MONTHS";
        let mut date = Date::new(date_string);
        date.plus(Date::month(2).as_str());
        assert_eq!(date.as_str(), expected);
    }

    #[test]
    fn test_date_minus_concat_text() {
        let date_string = "NOW";
        let expected = "NOW-2YEARS";
        let mut date = Date::new(date_string);
        date.minus(Date::year(2).as_str());
        assert_eq!(date.as_str(), expected);
    }

    #[test]
    fn test_range_create_inclusive_range() {
        let range = Range::inclusive("a", "b");
        assert_eq!(range.as_str(), "[a TO b]");
    }

    #[test]
    fn test_range_create_exclusive_range() {
        let range = Range::exclusive("a", "b");
        assert_eq!(range.as_str(), "{a TO b}");
    }
}