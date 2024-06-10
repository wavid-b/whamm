#[no_mangle]
pub fn add(mut a: i32, b: i32) -> i32 {
    a = a+1;
    let res = a + b;

    return res;
}

fn main() {
    let mut a = 0;
    let mut b = 3;
    let c = add(a, b);

    if c > 0 {
        b = 5;
    }
}