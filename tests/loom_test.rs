#[cfg(loom)]
use loom::sync::Arc;
#[cfg(loom)]
use loom::thread;
use rust_custom_channel::Buffer;

#[cfg(loom)]
#[test]
fn check_buffer() {
    loom::model(|| {
        let t = Buffer::<i32>::new(1);
        let arc = Arc::new(t);
    
        let mut in_thrs = vec![];
        let mut out_thrs = vec![];
        let loop_count = 1;
    
        for i in 0..loop_count {
            let tarc = arc.clone();
            in_thrs.push(thread::spawn(move || {
                tarc.push(i);
            }))
        }
    
        for _ in 0..loop_count {
            let tarc = arc.clone();
            out_thrs.push(thread::spawn(move || -> i32 {
                tarc.pop().unwrap()                
            }));
        }
    
        for t in in_thrs {
            t.join().unwrap();
        }

        let mut res = vec![];

        for t in out_thrs {
            res.push(t.join().unwrap());
        }

        assert_eq!(arc.clone().len(), 0);

        res.sort();
        assert_eq!(res, (0..loop_count).collect::<Vec<_>>());

        drop(arc);
    });
}