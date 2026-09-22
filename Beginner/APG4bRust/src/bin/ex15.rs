use proconio::input;

fn main() {
    input! {
        n: usize,
        s: i64,
        a: [i64;n],
        p: [i64;n],
    }

    let mut ans = 0;

    for i in 0..n {
        for j in 0..n {
            if a[i]+p[j] == s {
                ans += 1;
            }
        }
    }

    println!("{}", ans)
}