#[test]
fn test_hello_world() {
    assert_eq!(2 + 2, 4);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_add() {
        assert_eq!(super::add(2, 2), 4);
    }

    #[test]
    #[should_panic]
    fn test_fail() {
        assert!(false, "This test will fail.");
    }

    #[test]
    fn test_mul() -> Result<(), String> {
        if 2 * 2 == 4 {
            Ok(())
        } else {
            Err(String::from("two times two does not equal four"))
        }
    }
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
