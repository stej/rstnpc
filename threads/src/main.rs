use once_cell::sync::Lazy;
use std::{sync::Mutex, env::JoinPathsError};

static COUNTER: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(10));

#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
lazy_static! {
    static ref FRUITS: HashMap<u32, &'static str> = {
        println!("Initializing FRUITS");
        let mut m = HashMap::new();
        m.insert(1, "apple");
        m.insert(2, "banana");
        m
    };
}

static DATA: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    println!("Initializing DATA");
    "Hello, World!".to_string()
});

use static_init::dynamic;
#[dynamic]
static L1: Vec<i32> = vec![1, 2, 3, 4, 5, 6];

use std::{thread, vec, thread::JoinHandle};

fn main() {
    {
        let mut counter = COUNTER.lock().unwrap();
        *counter += 1;
    }
    println!("Counter: {:?}", COUNTER.lock());

    let x = 10;
    let ref_x = std::cell::RefCell::new(x);
    // Hodnota x je immutable, ale ref_x je mutable.
    *(ref_x.borrow_mut()) += 20;
    println!("{} {:?}", x, ref_x); // 20

    let x = std::cell::Cell::new(10);
    unsafe {
        *x.as_ptr() = 20;
    }
    x.set(30);
    println!("{}", x.get()); // 30

    let counter = Mutex::new(0); // mention poisoning
    {
        *counter.lock().unwrap() += 1;
    }
    println!("Counter: {:?}", *counter.lock().unwrap()); // 1

    let lock = std::sync::RwLock::new(String::from("Hello, "));
    {
        // Multiple read locks can be held at once.
        let read_guard1 = lock.read().unwrap();
        let read_guard2 = lock.read().unwrap();
        println!("Readers see: '{}{}'", *read_guard1, *read_guard2);
    }
    {
        // Only one write lock may be held, and no reads can occur simultaneously.
        let mut write_guard = lock.write().unwrap();
        write_guard.push_str("world!");
    }
    {
        let read_guard = lock.read().unwrap();
        println!("After modification: '{}'", *read_guard);
    }
    //============================
    let cell = std::cell::OnceCell::new();
    let value: &String = cell.get_or_init(|| "Hello, World!".to_string());
    let value: &String = cell.get_or_init(|| "NEW VALUE".to_string());
    assert_eq!(value, "Hello, World!");
    //============================
    println!("{:?}", FRUITS.get(&1)); // Some("apple")

    //===========================
    println!("{} {:?}", *DATA, DATA); // Hello, World!

    //===========================
    println!("{:?}", L1.len());
    //===========================
    let message = "Hello, World!".to_string();
    let handle = thread::spawn(move || {
        // let toprint = message;
        // println!("thread here: {}", toprint);
        println!("thread here: {}", &message);
    });
    handle.join().unwrap();
    ///println!("Printed message: {}", message);
    //===========================
    println!("{:?}", thread::current().id());
    //===========================
    let Err(e) = thread::spawn(|| {
        std::panic::panic_any(42);
    }).join() else { 
        panic!("The child thread panicked");
    };
    println!("Thread error: {:?}", e.downcast::<i32>().unwrap());
    //===========================
    (1..150)
        .into_iter()
        .map(|i| thread::spawn(move || {
            println!("Thread {}", i);
            thread::sleep(std::time::Duration::from_millis(i));
            i
        }))
        .for_each(|h| {
            let i = h.join().unwrap();
            println!("Thread {} finished", i);
        });


}
