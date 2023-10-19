use std::{error::Error, fmt::Display};
use std::fmt::Formatter;
use std::{fmt, vec};

pub struct Csv {
    header: Strings,
    rows: Vec<Strings>,
}

type Strings = Vec<String>;

impl Csv {
    pub fn parse_line(input: &str) -> Strings {
        input.split(',').map(String::from).collect()
    }
    pub fn parse(input: &str) -> Result<Csv, Box<dyn Error>> {
        let mut lines = input.lines();
        let header = Self::parse_line(lines.next().ok_or("no header")?);

        // parse
        let rows : Vec<Strings> = 
            lines
                .map(|line| { Self::parse_line(line) })
                .collect();
        // validate
        let all_size_as_header = rows
                            .iter()
                            .all(|row| row.len() == header.len());
        if !all_size_as_header {
            return Err("error parsing csv".into());
        }
        Ok(Csv{
            header: header,
            rows: rows,
        })
    }

    fn get_cols_widths(&self) -> Vec<usize> {
        self.header.iter().enumerate()
            .map(|(i, header)|
                std::cmp::max(
                    header.len(), 
                    self.rows.iter()
                        .map(|row| row[i].len())
                        .max()
                        .unwrap()))
            .collect()
    }

    fn stringify(&self) -> String {

        fn row_to_string(row: &Strings, cols_widths: &Vec<usize>) -> String {
            row.iter().enumerate()
                .map(|(i, col)| format!("{:width$}", col, width=cols_widths[i]))
                .collect::<Vec<String>>()
                .join("|")
        }

        fn get_header_separator(cols_widths: Vec<usize>) -> String {
            let width = 
                cols_widths.iter().sum::<usize>() +  // cell width
                cols_widths.len() - 1;       // separator
            String::from("-").repeat(width)
        }
        let cols_widths = Self::get_cols_widths(self);
        let header = row_to_string(&self.header, &cols_widths);
        let rows = 
            self.rows.iter()
                .map(|r| row_to_string(r, &cols_widths))
                .collect::<Vec<String>>()
                .join("\n");
        header + "\n" + &get_header_separator(cols_widths) + "\n" + &rows
    }
}

impl Display for Csv {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}