use proconio::input;

fn main() {
    input! {
        n: i64,
        a: [i64; n],
    }

    let avg: i64 = a.iter().sum::<i64>() / n;

    for &ai in &a {
        println!("{}", (ai - avg).abs());
    }
}