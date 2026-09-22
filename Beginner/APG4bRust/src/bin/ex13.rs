use proconio::input;

fn sum_scores(score: &Vec<i64>) -> i64 {
    return score.iter().sum();
}


fn output(sum_a:i64, sum_b:i64, sum_c:i64) {
    println!("{}", sum_a*sum_b*sum_c);
}


fn input_list(n:usize) -> Vec<i64> {
    input! {
        l: [i64;n],
    }
    return l;
}


fn main() {
    input! {
        n: usize,
    }
    let a = input_list(n);
    let b = input_list(n);
    let c = input_list(n);

    let sum_a = sum_scores(&a);
    let sum_b = sum_scores(&b);
    let sum_c = sum_scores(&c);

    output(sum_a,sum_b,sum_c)
}