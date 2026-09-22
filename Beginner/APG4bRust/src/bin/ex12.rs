use proconio::input;

fn main() {
    input! {
        n: i64,
        t: [i64;n],
    }

    let min = *t.iter().min().unwrap();
    println!("{}", t.iter().position(|&fast| fast == min).unwrap() + 1);
}