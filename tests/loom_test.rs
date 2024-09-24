#[cfg(loom)]
use loom::sync::Arc;
#[cfg(loom)]
use loom::sync::atomic::Ordering::{Acquire, Release, Relaxed};
#[cfg(loom)]
use loom::thread;
use rust_custom_channel::Buffer;

#[cfg(loom)]
#[test]
fn check_buffer() {
    loom::model(|| {
        let t = Buffer::<i32>::new(100);
        let arc = Arc::new(t);
    
        let mut thrs = vec![];
    
        for i in 0..10 {
            let tarc = arc.clone();
            thrs.push(thread::spawn(move || {
                // println!("Pushed {}", i);
                tarc.push(i);
            }))
        }
    
        for i in 0..10 {
            let tarc = arc.clone();
            thrs.push(thread::spawn(move ||{
                let v = tarc.pop();
    
                // match v {
                //     None => println!("extraced None for {} iter", i),
                //     Some(val) => println!("extracted {} for iter {}", val, i)
                // }
                
            }));
        }
    
        for t in thrs {
            t.join().unwrap();
        }

        assert_eq!(arc.clone().len(), 0);

        drop(arc);
    });
}