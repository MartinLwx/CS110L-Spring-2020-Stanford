fn drip_drop() -> &String {
    let s = String::from("hello world!");
    return &s;
    // the function will call Drop() finally, s will get destroyed
    // so we can't return the &s (there is no value for it to be borrowed from)
}
