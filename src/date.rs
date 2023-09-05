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
        self.date.push_str(&*("+".to_owned() + duration));
        self
    }

    pub fn minus(&mut self, duration: &str) -> &mut Self {
        self.date.push_str(&*("-".to_owned() + duration));
        self
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

    #[test]
    fn test_plus_concat_text() {
        let date_string = "NOW";
        let expected = "NOW+2MONTHS";
        let mut date = Date::new(date_string);
        date.plus(Date::month(2).as_str());
        assert_eq!(date.as_str(), expected);
    }

    #[test]
    fn test_minus_concat_text() {
        let date_string = "NOW";
        let expected = "NOW-2YEARS";
        let mut date = Date::new(date_string);
        date.minus(Date::year(2).as_str());
        assert_eq!(date.as_str(), expected);
    }

}