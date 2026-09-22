use proconio::input;

fn main() {
    input! {
        a: [i64;5],
    }

    if a[0] == a[1] || a[3] == a[4] {
        println!("YES");
        return
    }

    for i in 1..3 {
        if a[i] == a[i-1] || a[i] == a[i+1] {
            println!("YES");
            return
        }
    }

    println!("NO");
}