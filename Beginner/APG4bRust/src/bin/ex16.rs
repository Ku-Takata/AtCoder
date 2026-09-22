use proconio::input;

fn main() {
    input! {
        n: usize,
        k: usize,
        s: [String;n],
    }

    for i in 0..n {
        if s[i].len() >= k {
            print!("{} ", s[i]);
        }
    }
    println!();
}