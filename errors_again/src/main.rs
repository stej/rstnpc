struct Bar;
trait Foo {
    fn f(&self);
}
impl Foo for Bar {
    fn f(&self) {
        println!("Bar.f()");
    }
}

//---------------------------
use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use std::io;
use std::os::windows::process;

#[derive(Debug)]
struct MyError {
    message: String,
    inner_error: Option<io::Error>,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError: {}", self.message)
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.inner_error.as_ref().map(|e| e as &dyn Error)
    }
    // no compile
    // fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
    //     // Logic to return a backtrace if availabl
    // }
}

use anyhow::{anyhow, bail, Context, Result};
fn main() -> Result<()> {
    println!("Hello, world!");
    let b = Bar;
    <Bar as Foo>::f(&b);

    //-------------------------------
    let bck = Backtrace::force_capture();

    //---------------------------
    fn might_fail(flag: bool) -> Result<()> {
        if flag {
            Ok(())
        } else {
            Err(anyhow!("it failed")) // return anyow::Error
        }
    }
    //might_fail(false)?;

    //---------------------------
    // std::fs::read_to_string("foo.txt")
    //     .context("failed to read foo.txt")?;

    //-------------------------------
    //bail!("Something went wrong");
    //---------------------------
    //anyhow::ensure!(1== 2, "not equal: {}", 1);
    //-------------------------------
    fn task1() -> Result<()> {
        Err(anyhow!("Task 1 failed"))
    }
    fn task2() -> Result<()> {
        task1().with_context(|| "Task 2 failed while executing task 1")
    }
    match task2() {
        Ok(_) => println!("Success"),

        Err(err) => {
            eprintln!("Error: {:?}", err);

            for cause in err.chain() {
                eprintln!("!Caused by: {:?}", cause);
            }
        }
    }
    //-------------------------------
    #[cfg(debug_assertions)]
    println!("should be called in debug mode only. Or if it's set to true in toml for release.");

    //---------------------------
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum DataProcessingError {
        #[error("Data not found: {0}")]
        NotFound(String),
        #[error("Invalid data format")]
        InvalidFormat,
        #[error("IO error")]
        Io(#[from] std::io::Error),
        #[error("Struct like!, name: {name}, id: {id}")]
        StructLikeError {name: String, id: i32}
    }

    fn process_data(file_path: &str) -> Result<(), DataProcessingError> {
        if file_path.is_empty() {
            return Err(DataProcessingError::NotFound(file_path.to_string()));
        }
        let data = std::fs::read_to_string(file_path)?;
        if data.is_empty() {
            return Err(DataProcessingError::InvalidFormat);
        }
        println!("Data processed: {}", data);
        Ok(())
    }
    //let res: anyhow::Result<()> = process_data("foo.txt"); ? nejde zkompilovat
    let res = process_data("foo.txt");
    println!("{:?}", res);

    Ok(())
}
