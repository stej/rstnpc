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
    pub is_interactive_operation: bool
}

impl OperationWithParam {
    pub fn from_cmdline(operation: Operation, param: String) -> Self {
        Self { operation, param, is_interactive_operation: false }
    }
    pub fn from_interactive(operation: Operation, param: String) -> Self {
        Self { operation, param, is_interactive_operation: true }
    }
    pub fn exit() -> Self {
        Self::from_interactive(Operation::Exit, String::new())
    }
}

#[cfg(test)]
mod tests_enum_fromstr {

    use crate::Operation;

    #[test]
    fn parse_enum() {
        assert_eq!(Ok(Operation::Len), "len".parse());
    }
}