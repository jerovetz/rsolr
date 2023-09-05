pub struct Date {
    date: String
}

impl Date  {

    pub fn new(date_string: &str) -> Self {
        Date { date: date_string.to_owned() }
    }

    pub fn as_str(&self) -> &str {
        &self.date
    }

}

#[cfg(test)]
mod tests {
    use crate::date::Date;

    #[test]
    fn test_as_str_returns_date() {
        let date_string = "NOW";
        assert_eq!(Date::new(date_string).as_str(), date_string);
    }

}