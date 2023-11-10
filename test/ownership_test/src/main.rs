#[derive(Debug)]
struct SomeStruct(i32);

fn remove_half_and_ret(v: &mut Vec<SomeStruct>) -> Vec<SomeStruct> {
    if v.len() % 2 != 0 {
        panic!("not even");
    }
    let second_half = v.split_off(v.len() / 2);
    //v.truncate(v.len() / 2);

    return second_half;
}
fn main() {
    let mut v = vec![SomeStruct(1), SomeStruct(2), SomeStruct(3), SomeStruct(4)];
    let half = remove_half_and_ret(&mut v);
    println!("{:?}", v);
    println!("{:?}", half);
}
