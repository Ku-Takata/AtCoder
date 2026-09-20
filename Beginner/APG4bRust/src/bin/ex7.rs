fn main() {
    let a: bool = true;
    let b: bool = false;
    let c: bool = true;

    if a {
        print!("At")
    }
    else {
        print!("Yo")
    }

    if a && b {
        print!("Bo")
    }
    else {
        print!("Co")
    }

    if a && b && c {
        print!("foo!")
    }
    else if true && false {
        print!("year!")
    }
    else if !a || c {
        print!("der")
    }

    println!("")
}