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

    pub fn as_str(&self) -> String {
        if self.mode == Type::Inclusive {
            return format!("[{} TO {}]",self.from, self.to);
        }

        format!("{{{} TO {}}}",self.from, self.to)
    }

}

#[cfg(test)]
mod tests {
    use crate::range::Range;

    #[test]
    fn test_create_inclusive_range() {
        let range = Range::inclusive("a", "b");
        assert_eq!(range.as_str(), "[a TO b]");
    }

    #[test]
    fn test_create_exclusive_range() {
        let range = Range::exclusive("a", "b");
        assert_eq!(range.as_str(), "{a TO b}");
    }

}