use proconio::marker::Chars;
use proconio::input;

fn main() {
    input! {
        s: Chars,
    }

    let mut ans = 1;

    for i in (1..s.len()).step_by(2) {
        if s[i] == '+' {
            ans += 1;
        }
        else {
            ans -= 1;
        }
    }

    println!("{}", ans);
}