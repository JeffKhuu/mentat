use regex::Regex;

/// NumericalString is a string that can only represent strings that are valid numbers.
/// Ex. "123", "14.5", "3.14151"
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct NumericalString(String);

impl ToString for NumericalString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl From<i64> for NumericalString {
    fn from(value: i64) -> Self {
        NumericalString(value.to_string())
    }
}
impl From<f64> for NumericalString {
    fn from(value: f64) -> Self {
        NumericalString(value.to_string())
    }
}

impl NumericalString {
    pub(crate) fn create(s: &str) -> Option<NumericalString> {
        let re = Regex::new(r"\b\d+(?:\.\d+)?\b").unwrap();
        if re.is_match(s) {
            return Some(NumericalString(s.to_string()));
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("2")]
    #[case("3.14159")]
    #[case("2.0")]
    #[case("0.5")]
    #[case("123.45")]
    fn test_numeric_string(#[case] raw: &str) {
        assert!(NumericalString::create(raw).is_some());
    }
    #[rstest]
    #[case("pi")]
    #[case("x+y")]
    #[case("sin(y)")]
    #[case("123abc")]
    fn test_nonnumeric_string(#[case] raw: &str) {
        assert!(NumericalString::create(raw).is_none());
    }
}
