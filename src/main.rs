use rust_custom_channel::Buffer;
use std::{thread, vec};
use std::sync::{Arc, Condvar};

fn main() {
    // let mut push_handers = vec![];
    // let mut pop_handlers = vec![];

    let t = Buffer::<i32>::new(3);
    let arc = Arc::new(t);

    let mut thrs = vec![];

    for i in 0..1000 {
        let tarc = arc.clone();
        thrs.push(thread::spawn(move || {
            println!("Pushed {}", i);
            tarc.push(i);
        }))
    }

    for i in 0..1000 {
        let tarc = arc.clone();
        thrs.push(thread::spawn(move ||{
            let v = tarc.pop();

            match v {
                None => println!("extraced None for {} iter", i),
                Some(val) => println!("extracted {} for iter {}", val, i)
            }
            
        }));
    }

    for t in thrs {
        t.join().unwrap();
    }
}
