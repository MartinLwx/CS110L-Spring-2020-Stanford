fn main() {
    let s1 = String::from("hello");
    let mut v = Vec::new();
    v.push(s1);
    // the s2 shouldn't take the ownership of v[0] (String)
    // it should borrow the v[0]
    // so we should use &v[0] instead of v[0]
    let s2: String = v[0];
    println!("{}", s2);
}
