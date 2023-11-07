use std::fmt;
use std::fmt::Formatter;
use std::{error::Error, fmt::Display};

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
        let rows: Vec<Strings> = lines.map(|line| Self::parse_line(line)).collect();

        // validate
        let any_header_empty: bool = header.iter().any(|h| h.is_empty());
        if any_header_empty {
            return Err("error parsing csv - empty header".into());
        }
        let all_size_as_header = rows.iter().all(|row| row.len() == header.len());
        if !all_size_as_header {
            return Err("error parsing csv - incorrect cols count".into());
        }
        if rows.is_empty() {
            eprintln!("warning: empty csv");
        }
        Ok(Csv { header, rows })
    }

    fn get_cols_widths(&self) -> Vec<usize> {
        self.header
            .iter()
            .enumerate()
            .map(|(i, header)| {
                std::cmp::max(
                    header.len(),
                    self.rows.iter().map(|row| row[i].len()).max().unwrap_or(0),
                )
            })
            .collect()
    }

    fn stringify(&self) -> String {
        fn row_to_string(row: &Strings, cols_widths: &[usize]) -> String {
            row.iter()
                .enumerate()
                .map(|(i, col)| format!("{:width$}", col, width = cols_widths[i]))
                .collect::<Vec<String>>()
                .join("|")
        }

        fn get_header_separator(cols_widths: &[usize]) -> String {
            let width = cols_widths.iter().sum::<usize>() +  // cell width
                cols_widths.len()
                - 1; // separator
            String::from("-").repeat(width)
        }
        let cols_widths = Self::get_cols_widths(self);
        let header = row_to_string(&self.header, &cols_widths);
        let rows = self
            .rows
            .iter()
            .map(|r| row_to_string(r, &cols_widths))
            .collect::<Vec<String>>()
            .join("\n");
        header + "\n" + &get_header_separator(&cols_widths) + "\n" + &rows
    }
}

impl Display for Csv {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}

#[cfg(test)]
mod tests_parse {
    use crate::csv::Csv;

    #[test]
    fn invalid() {
        assert_eq!(true, Csv::parse("").is_err());
        assert_eq!(true, Csv::parse("a,").is_err());
        assert_eq!(true, Csv::parse("a,\n").is_err());
        assert_eq!(true, Csv::parse(",a\n").is_err());
        assert_eq!(true, Csv::parse("a,b\n1").is_err());
        assert_eq!(true, Csv::parse("a\n1,2").is_err());
        assert_eq!(true, Csv::parse("a\n1\n1,2").is_err());
        assert_eq!(true, Csv::parse("a\n1,2\n1").is_err());
    }

    #[test]
    fn simplest_valid() {
        assert_eq!(true, Csv::parse("a").is_ok());
        assert_eq!(true, Csv::parse("a\n").is_ok());
        assert_eq!(true, Csv::parse("a,b").is_ok());
        assert_eq!(true, Csv::parse("a,b\n").is_ok());
    }

    #[test]
    fn simple_1col() {
        let csv = Csv::parse("a").unwrap();
        assert_eq!(vec!["a"], csv.header);
        assert!(csv.rows.is_empty());

        let csv = Csv::parse("a\nb").unwrap();
        assert_eq!(vec!["a"], csv.header);
        assert_eq!(vec![vec!["b"]], csv.rows);

        let csv = Csv::parse("a\nb\nc").unwrap();
        assert_eq!(vec!["a"], csv.header);
        assert_eq!(vec![vec!["b"], vec!["c"]], csv.rows);
    }

    #[test]
    fn simple_2col() {
        let csv = Csv::parse("a,a2").unwrap();
        assert_eq!(vec!["a", "a2"], csv.header);
        assert!(csv.rows.is_empty());

        let csv = Csv::parse("a,a2\nb,b2").unwrap();
        assert_eq!(vec!["a", "a2"], csv.header);
        assert_eq!(vec![vec!["b", "b2"]], csv.rows);

        let csv = Csv::parse("a,a2\nb,b2\nc,c2").unwrap();
        assert_eq!(vec!["a", "a2"], csv.header);
        assert_eq!(vec![vec!["b", "b2"], vec!["c", "c2"]], csv.rows);
    }

    #[test]
    fn simple_2col_empty_cell() {
        let csv = Csv::parse("a,a2\nb,").unwrap();
        assert_eq!(vec!["a", "a2"], csv.header);
        assert_eq!(vec![vec!["b", ""]], csv.rows);

        let csv = Csv::parse("a,a2\n,b").unwrap();
        assert_eq!(vec!["a", "a2"], csv.header);
        assert_eq!(vec![vec!["", "b"]], csv.rows);
    }

    #[test]
    fn simple_3col_empty_cell() {
        let csv = Csv::parse("a,a2,a3\nb,,b3").unwrap();
        assert_eq!(vec!["a", "a2", "a3"], csv.header);
        assert_eq!(vec![vec!["b", "", "b3"]], csv.rows);

        let csv = Csv::parse("a,a2,a3\n,,").unwrap();
        assert_eq!(vec!["a", "a2", "a3"], csv.header);
        assert_eq!(vec![vec!["", "", ""]], csv.rows);
    }
}

#[cfg(test)]
mod tests_display {
    use crate::csv::Csv;

    #[test]
    fn simplest() {
        assert_eq!("a\n-\n", Csv::parse("a").unwrap().to_string());
        assert_eq!("a\n-\nb", Csv::parse("a\nb").unwrap().to_string());
    }

    #[test]
    fn simple_2col() {
        let display = "\
          a|b\
        \n---\
        \nc|d";
        assert_eq!(display, Csv::parse("a,b\nc,d").unwrap().to_string());

        let display = "\
          a|blong\
        \n-------\
        \nc|d    ";
        assert_eq!(display, Csv::parse("a,blong\nc,d").unwrap().to_string());

        let display = "\
          along|b\
        \n-------\
        \nc    |d";
        assert_eq!(display, Csv::parse("along,b\nc,d").unwrap().to_string());

        let display = "\
          along|blong\
        \n-----------\
        \nc    |d    ";
        assert_eq!(display, Csv::parse("along,blong\nc,d").unwrap().to_string());

        let display = "\
          a    |b\
        \n-------\
        \nclong|d";
        assert_eq!(display, Csv::parse("a,b\nclong,d").unwrap().to_string());

        let display = "\
          a|b    \
        \n-------\
        \nc|dlong";
        assert_eq!(display, Csv::parse("a,b\nc,dlong").unwrap().to_string());
    }

    #[test]
    fn more_columns() {
        let display = "\
          asome  |b |dsome        \
        \n------------------------\
        \nc      |d |adf          \
        \nsomelon|ab|             \
        \n       |  |somelongagain\
        \na      |b |c            ";
        assert_eq!(
            display,
            Csv::parse("asome,b,dsome\nc,d,adf\nsomelon,ab,\n,,somelongagain\na,b,c")
                .unwrap()
                .to_string()
        );
    }
}
