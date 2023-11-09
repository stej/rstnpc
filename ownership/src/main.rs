// this does not compile in Rust
// fn i32_as_str(number: &i32) -> &str {
//     // compute the string
//     let s = format!("{}", number);
//     // We are returning a reference to something that
//     // exists only in i32_as_str() - bad
//     // technical term would be “returning a dangling pointer”
//     // and doing a “use after free”
//     &s
// }

#[derive(Debug)]

struct X<'a>(&'a i32);
impl Drop for X<'_> {
    fn drop(&mut self) {}
}

impl<'a> MyTrait<'a> for X<'a> {
    fn foo(&self) -> &'a i32 {
        self.0
    }
}

fn fun_fn(x: fn() -> String) {
    let mut s = String::new();
    let x = || ();
    let y = || s.clear();
    let z = move || s;
}

fn print_2_numbers<'a, 'b>(x: &'a i32, y: &'b i32) {
    println!("x is {}, y is {}", x, y);
}

fn cut<'a>(input: &'a str, num: usize) -> &'a str {
    &input[num..]
}

fn my_method<'self, 'a>(&'self self, other_str: &'a str) -> &'self str {}

fn main() {
    println!("Hello, world!");

    let x = 0;
    let z;
    let y = &x; // y should die before z if lifetime would not be larger
    z = y;
    println!("z: {}", z);

    //-----------------------------
    let mut data = vec![1, 2, 3];
    let x = &data[0];
    println!("{}", x);
    // This is OK, x is no longer needed
    data.push(4);

    //-----------------------------
    let mut data = vec![1, 2, 3];
    let x = X(&data[0]);
    println!("{:?}", x);
    drop(x);
    data.push(4);
    // Here, the destructor is run and therefore this'll fail to compile.
    // The only way to convince the compiler that ‘x’ is no longer valid is to drop it explicitly with mem::drop()

    //--------------------------------
    let mut data = vec![1, 2, 3];

    // This mut allows us to change where the reference points to
    let mut x = &data[0];
    println!("{}", x); // Last use of this borrow
    data.push(4); // this is valid because we will no longer refer to &data[0] via x
    x = &data[3]; // We start a new borrow here
    println!("{}", x);

    //--------------------------------
    fn get_str<'a>(s: *const String) -> &'a str {
        unsafe { &*s }
    }
    let soon_dropped = String::from("hello");
    let dangling = get_str(&soon_dropped);
    drop(soon_dropped);
    println!("Invalid str: {}", dangling); // Invalid str: gӚ_`

    //--------------------------------
    fn debug<'a>(a: &'a str, b: &'a str) {
        println!("a = {a:?} b = {b:?}");
    }
    let hello: &'static str = "hello";
    {
        let world = String::from("world");
        let world = &world; // 'world has a shorter lifetime than 'static
        debug(hello, world); // hello silently downgrades from `&'static str` into `&'world str`
    }
}
