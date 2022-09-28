use crossbeam_channel;
use std::{thread, time};
use std::process::id;


fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    // Note: use the default value to resize the vector, so that we can avoid index error later
    output_vec.resize_with(input_vec.len(), Default::default);

    // TODO: implement parallel map!
    // two channels: one for arguments, one for results
    let (sender, receiver) = crossbeam_channel::unbounded();
    let (res_sender, res_receiver) = crossbeam_channel::unbounded();

    // step 1. create num_threads to wait for the data to come in
    let mut handles = vec![];
    for _ in 0..num_threads {
        let another_res_sender = res_sender.clone();
        let another_receiver = receiver.clone();
        handles.push(thread::spawn(move || {
            while let Ok((msg, index)) = another_receiver.recv() {
                another_res_sender.send((f(msg), index)).unwrap();
            }
        }))
    }

    // step 2. send arguments to threads
    while let Some(val) = input_vec.pop() {
        // idea: because we always pop value from the vector
        // , the index of the last element is equal to length - 1
        // be cause we pass the msg after .pop(), so we can just send the current length
        sender.send((val, input_vec.len()))
              .expect("Tried writing to channel, but there are no receivers!");
    }

    // step 3. notify the receivers that there are no more msg!
    drop(sender);
    drop(res_sender);

    // step 4. wait for every thread
    for handle in handles {
        handle.join().unwrap();
    }

    // step 5. collect results
    for (res, index) in res_receiver.iter() {
        output_vec[index] = res;
    }

    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let start = time::Instant::now();
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("Time consumed: {:?}", start.elapsed());
    println!("squares: {:?}", squares);
}
