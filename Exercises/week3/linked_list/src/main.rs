use crate::linked_list::ComputeNorm;
use linked_list::LinkedList;

pub mod linked_list;

fn main() {
    // test the basics
    println!("[Test for the basis]");
    let mut list: LinkedList<String> = LinkedList::new();
    let mut another_list: LinkedList<String> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    list.push_front(String::from("hello"));
    another_list.push_front(String::from("hello"));
    assert!(list == another_list);
    list.push_front(String::from("world"));
    assert!(list != another_list);
    println!("==> list:        {}", list);
    println!("==> list size:   {}", list.get_size());
    println!("==> top element: {}", list.pop_front().unwrap());
    println!("==> list:        {}", list);
    println!("==> size:        {}", list.get_size());
    println!("==> list:        {}", list.to_string()); // ToString impl for anything impl Display

    // test the clone trait
    println!("[Test for the Clone trait]");
    let cloned_list = list.clone();
    println!("==> the cloned list: {}", cloned_list);

    // test the ComputeNorm trait
    println!("[Test for the ComputeNorm trait]");
    let mut list = LinkedList::new();
    list.push_front(3.0);
    list.push_front(4.0);
    println!("==> current list: {}", list);
    println!("==> the L2 Norm: {}", list.compute_norm());
}
