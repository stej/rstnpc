use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Operation {
    Lowercase,
    Uppercase,
    Slugify,
    NoSpaces,
    Len,
    Reverse,
    Csv,
    Exit
}

impl FromStr for Operation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lowercase" => Ok(Operation::Lowercase),
            "uppercase" => Ok(Operation::Uppercase),
            "slugify" => Ok(Operation::Slugify),
            "no-spaces" => Ok(Operation::NoSpaces),
            "len" => Ok(Operation::Len),
            "reverse" => Ok(Operation::Reverse),
            "csv" => Ok(Operation::Csv),
            _ => Err(format!("Unknown operation: {}", s))
        }
    }
}

#[derive(Debug)]
pub struct OperationWithParam {
    pub operation: Operation,
    pub param: String,
}

#[cfg(test)]
mod tests_enum_fromstr {

    use crate::Operation;

    #[test]
    fn parse_enum() {
        assert_eq!(Ok(Operation::Len), "len".parse());
    }
}