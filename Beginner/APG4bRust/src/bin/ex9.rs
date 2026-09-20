use proconio::input;

fn main() {
    input! {
        n: i64,
        a: i64,
        query: [(char,i64); n],
    }

    let mut a = a;

    for (i, (op,b)) in (1..=n).zip(query) {
        match op {
        '+' => {
            a += b;
            println!("{i} {a}");
        }
        '-' => {
            a -= b;
            println!("{i} {a}");
        }
        '*' => {
            a *= b;
            println!("{i} {a}");
        }
        '/' => {
            if b != 0 {
                a = a.div_euclid(b);
                println!("{i} {a}");
            }
            else {
                println!("error");
                break
            }
        }
        _ => {
            println!("error");
            break
        }
        }
    }
}