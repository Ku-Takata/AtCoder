use proconio::input;

fn main() {
    input! {
        a: i64,
        op: char,
        b: i64,
    }

    if op == '+' {
        println!("{}", a+b);
    }
    else if op == '-' {
        println!("{}", a-b);
    }
    else if op == '*' {
        println!("{}", a*b);
    }
    else if op == '/' && b != 0 {
        println!("{}", a/b)
    }
    else {
        println!("error")
    }
}