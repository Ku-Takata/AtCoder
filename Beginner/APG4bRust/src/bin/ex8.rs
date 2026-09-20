use proconio::input;

fn main() {
    input! {
        n: i64,
        ab: [(i64,i64); n],
    }

    for (a,b) in ab {
        println!("{}", a+b);
    }
}