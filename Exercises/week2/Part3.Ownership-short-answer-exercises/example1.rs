fn main() {
    let mut s = String::from("hello");
    let ref1 = &s;       // ref1 borrow from s
    let ref2 = &ref1;    // ref2 borrow from ref1
    let ref3 = &ref2;    // ref3 borrown from ref2
    // we use ref3 in the println!("{}", ref3.to_uppercase())
    // so the ownership doesn't return to s here yet.
    // as a result, we can't assign the strint to s
    s = String::from("goodbye");
    println!("{}", ref3.to_uppercase());
}
