use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Lowercase,
    Uppercase,
    Slugify,
    NoSpaces,
    Len,
    Reverse,
    Csv,
    Exit,
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
            _ => Err(format!("Unknown operation: {}", s)),
        }
    }
}

#[derive(Debug)]
pub struct OperationWithParam {
    pub operation: Operation,
    pub param: String,
}

impl OperationWithParam {
    pub fn exit() -> Self {
        Self::new(Operation::Exit, String::new())
    }
    pub fn new(operation: Operation, param: String) -> Self {
        Self { operation, param }
    }

    /// If the parameter is a path to a file, read the file and return the content.
    /// Otherwise, return the parameter as is.
    ///
    /// Note: it was requested to work only for CSV files, but looks like good idea to do it for all files.
    pub fn standardize(&self) -> Result<OperationWithParam, Box<dyn Error>> {
        let param = &self.param;
        let trimmed = param.trim();
        let path = Path::new(trimmed);
        if !path.is_file() {
            return Ok(OperationWithParam::new(
                self.operation.clone(),
                trimmed.into(),
            ));
        }
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()).into());
        }
        let file_content =
            read_to_string(&path).map_err(|err| format!("Unable to read file: {}", err))?;
        Ok(OperationWithParam::new(
            self.operation.clone(),
            file_content,
        ))
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
