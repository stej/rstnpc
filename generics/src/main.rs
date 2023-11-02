fn merge<T, U>(first: T, second: U) -> (T, U) {
    (first, second)
}

fn compare<T, U>(_first: T, _second: U)
where
    T: PartialOrd + Copy,
    U: PartialOrd + Copy,
{
}

struct Array<T, const N: usize> {
    elements: [T; N],
}

fn longest<'a, T>(x: &'a T, y: &'a T) -> &'a T
where
    T: PartialOrd,
{
    if x > y {
        x
    } else {
        y
    }
}

struct RefHolder<'a, T> {
    ref_to_t: &'a T,
}

trait MyTrait {
    const SOME_CONST: usize;
    type MyType: std::fmt::Debug;
    fn do_something();
    fn optional_method() {
        println!("default impl");
    }
}
struct MyType;
impl MyType {
    const SOME_CONST: usize = 42;
    fn do_something() {
        println!("MyType impl MyTrait");
    }
}

struct MyTypeAdd;
impl std::ops::Add<u32> for MyTypeAdd {
    type Output = f32;
    fn add(self, rhs: u32) -> Self::Output {
        rhs as f32
    }
}

trait Describable {
    fn describe(&self) -> String;
}
impl Describable for String {
    fn describe(&self) -> String {
        format!("String: {}", *self)
    }
}

fn main() {
    let _integers: Array<i32, 5> = Array {
        elements: [1, 2, 3, 4, 5],
    };

    let _floats: Array<f32, 3> = Array {
        elements: [1.1, 2.2, 3.3],
    };
    //------------------------------------
    let string1 = String::from("Rust");
    let result;

    {
        let _string2 = String::from("C++");
        let holder = RefHolder { ref_to_t: &string1 };
        result = holder.ref_to_t;
        // 'result' lives as long as 'string1'
    }
    //------------------------------------
    fn print_description(item: &impl Describable) {
        println!("{}", item.describe());
    }

    // fn print_description<T: Describable>(item: &T) {
    //     println!("{}", item.describe());
    // }
    // these are identical, except in the first case,
    // the generic param can only be inferred and cannot
    // be written in manually

    //------------------------------------
    // existensional types
    fn my_func() -> impl Describable {
        String::from("Hello")
    }
    //------------------------------------
    fn my_func_closure() -> impl Fn() -> i32 {
        || 12
    }
    let _num = my_func_closure()();
    //------------------------------------
    // tady je vystupem neco, co je definovano z method body
    fn get_iter() -> impl Iterator<Item = i32> {
        (0..12).map(|x| x * 2)
    }
    //------------------------------------
    // generics jsou v rustu monomorphizovane; kopirovany pro kazdej typ
    //
    // toto je dynamicky dispatch
    // mensi ve velikosti, ale trochu pomalejsi
    fn print_dynamic(item: &dyn Describable) {
        println!("{}", item.describe());
    }
    // drive slo toto
    // fn print_dynamic2(item: &Describable) {
    //     println!("{}", item.describe());
    // }
    // trait nemuze pozadovat, ze typ je Sized, ale funkce muzou

    trait ObjectUnsafe {
        fn new_instance(&self) -> Self
        where
            Self: Sized,
        {
            unimplemented!()
        }
    }
    fn try_trait_objective(_: &dyn ObjectUnsafe) {}
    //------------------------------------
    trait Display: ToString {
        fn display(&self);
        //fn display(&self) { println!("{}", self.to_string()); }
    }
    struct Product {
        id: u32,
        name: String,
    }

    impl ToString for Product {
        fn to_string(&self) -> String {
            format!("Product ID: {}", self.id)
        }
    }
    impl Display for Product {
        fn display(&self) {
            println!("{}", self.to_string());
        }
    }
    //------------------------------------
    fn apply_to_all<'a, F>(f: F, items: &[&'a str])
    where
        // every type F that is closure that takes string slice and the lifetime is 'b
        F: for<'b> Fn(&'b str),
    {
        for &item in items {
            f(item);
        }
    }

    let echo: for<'a> fn(&'a str) = |x| println!("{}", x);
    apply_to_all(echo, &["Hello", "World"]);
    // je to v pripadech generik pred iteratory, nebo pres .. (closures?)

    //------------------------------------
    //T: From<U>
    // dostavam --> U: Into<T> zadara
    struct StringContainer(String);
    impl From<String> for StringContainer {
        fn from(item: String) -> Self {
            Self(item)
        }
    }
    // mam tedy StringContainer::from(String::new())
    // ale i String::new().into()
    let c =StringContainer::from(String::new());
    let sc : StringContainer = String::new().into();

    //------------------------------------
    // let fibo = Fibonacci::<u64>::new();

    // fn next(&mut self) -> Option<Self::Item> {
    //     let newnext = self.curr.checked_add(self.next)?;
    // }
    //------------------------------------

    struct Square { 
        side: f64
    }
    struct Circle { 
        radius: f64
    }
    trait Drawable {
        fn draw(&self);
    }
    impl Drawable for Circle {
        fn draw(&self) {
            println!("Drawing a circle");
        }
    }
    impl Drawable for Square {
        fn draw(&self) {
            println!("Drawing a square");
        }
    }
    let shares: Vec<Box<dyn Drawable>> = vec![Box::new(Circle{radius:5.3}), Box::new(Square{side:5.5})];
    for share in shares {
        share.draw();
    }  
    //------------------------------------
    // use std::any::Any; pro downcast
    // downcast
}
