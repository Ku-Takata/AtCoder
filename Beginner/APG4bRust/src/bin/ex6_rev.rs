use proconio::input;

fn main() {
    input! {
        a: i64,
        op: char,
        b: i64,
    }

    match op {
        '+' => println!("{}", a+b),
        '-' => println!("{}", a-b),
        '*' => println!("{}", a*b),
        '/' => {
            if b != 0 {
                println!("{}", a/b);
            }
            else {
                println!("error")
            }
        }
        _ => println!("error")
    }
}