#[cfg(loom)]
use loom::sync::Arc;
#[cfg(loom)]
use loom::sync::atomic::Ordering::{Acquire, Release, Relaxed};
#[cfg(loom)]
use loom::thread;
use rust_custom_channel::Buffer;

#[cfg(loom)]
#[test]
fn check_abuffer() {
    use rust_custom_channel::abuffer::ABuffer;

    loom::model(|| {
        let t = ABuffer::<i32>::new(1);
        let arc = Arc::new(t);
    
        let mut thrs_in = vec![];
        let mut thrs_out = vec![];

        let loop_count = 1;
    
        for i in 0..loop_count {
            let tarc = arc.clone();
            thrs_in.push(thread::spawn(move || {
                assert_eq!(tarc.try_push(i), Ok(()));
            }))
        }
    
        for i in 0..loop_count {
            let tarc = arc.clone();
            thrs_out.push(thread::spawn(move || ->i32 {
                let mut val = tarc.try_pop();
                while val.is_none() {
                    val = tarc.try_pop();
                    thread::yield_now();
                }
                val.unwrap()
            }));
        }
    
        for t in thrs_in {
            t.join().unwrap();
        }

        let mut res = thrs_out.into_iter().map(|t| t.join().unwrap()).collect::<Vec<_>>();

        assert_eq!(arc.len(), 0);

        res.sort();
        assert_eq!(res, (0..loop_count).collect::<Vec<_>>());

        drop(arc);
    });
}