#![feature(test)]

// cucumber
// Cargo-dinghy
// proptest - prooperty testing
// mockall - mock testing
// rstest - fixture-based testing
// cargo-mutants - mutation testing
// mutagen - mutation testing

// cargo tarpaulin
// cargo tarpaulin --fail-under 80

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello_world_attr() {
        assert_eq!(2 + 2, 4);
    }
}

// hyperfine

extern crate test;

pub fn some_function() {

    // Function whose performance you want to test

    // For example, a sorting algorithm, a calculation, etc.
}

#[cfg(test)]
mod tests2 {
    use super::*;

    use test::Bencher;

    #[bench]
    fn bench_some_function(b: &mut Bencher) {
        b.iter(|| {
            // Call the function inside `iter` to benchmark it

            some_function();
        });
    }
}
