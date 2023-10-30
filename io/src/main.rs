use std::io::Cursor;
use std::io::{self};

fn main() {
    let data = [1, 2, 3, 4, 5];
    let mut cursor = Cursor::new(&data);
    let mut buffer = [0u8; 3];
    cursor.read(&mut buffer).unwrap();
    println!("Buffer: {:?}", buffer);
    //--------------------
    println!("{:015}, {:x}", 345.41, 2505);
    //--------------------
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    println!("You entered: {}", input.trim());
    //--------------------
    let num: i32 = "123".trim().parse().expect("should be num");
    println!("num: {}", num);
    //--------------------
    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::open("testfile.txt").expect("Unable to open the file");
    println!("{:?}", file);
    drop(file);
    //--------------------
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open("testfile.txt")
        .expect("Unable to open the file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Unable to read the file");
    println!("File content: \n{}", content);
    //--------------------
    //sometimes print!(..) doesnt flush the output to the console
    std::io::stdout().flush().unwrap();

    //--------------------
    // raw string
    use std::io::BufWriter;
    let file = File::create("testfile2.txt").expect("Unable to create the file");
    let mut writer = BufWriter::new(file);
    writer
        .write_all(b"Hello, Buffered Rust!")
        .expect("Unable to write to the file");

    //--------------------
    //??????????????????????????
    use std::fs;
    println!("{:?}", fs::read_to_string("testfile2.txt").unwrap());
    //--------------------
    use std::path::{Path, PathBuf};
    let path = Path::new("/path/to/file.txt");
    let mut path_buf = PathBuf::from("path");
    path_buf.push("to");
    path_buf.push("file.txt");
    println!("{}", path_buf.display());

    //------------------------
    // Create a new directory
    fs::create_dir("new_directory").expect("Failed to create directory");
    // Read a directory's contents
    for entry in fs::read_dir("new_directory").expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        println!("{:?}", entry.path());
    }
    // Remove a directory
    fs::remove_dir("new_directory").expect("Failed to remove directory");

    //------------------------
    let metadata = fs::metadata("testfile.txt").expect("Failed to fetch metadata");
    println!("Is it a file? {}", metadata.is_file());
    println!("Is it a directory? {}", metadata.is_dir());
    println!("File size: {}", metadata.len());
    let permissions = metadata.permissions();
    println!("Read-only? {}", !permissions.readonly());
    println!("all: {:?}", permissions);
}
