use core::sync;
use std::{fmt, ptr};
use std::sync::Arc;

const POSITIVE_INF: i32 = i32::MAX;
const NEGATIVE_INF: i32 = i32::MIN;


#[derive(Debug)]
struct NodePtr {
    flag: bool,
    mark: bool,
    thread: bool,
    node_ref: Arc<Node>,
}

#[derive(Debug)]
struct Node {
    k: i32,
    child: [ sync::atomic::AtomicPtr<NodePtr>; 2],
    back_link: sync::atomic::AtomicPtr<Node>,
    pre_link: sync::atomic::AtomicPtr<Node>,
}

struct Bst {
    root: [Arc<Node>; 2],
    optimization: bool,
}

impl fmt::Debug for Bst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bst {{ root: {:?}, optimization: {} }}", self.root, self.optimization)
    }
}


impl Node {
    fn new(k: i32, child: [sync::atomic::AtomicPtr<NodePtr>; 2], back_link: sync::atomic::AtomicPtr<Node>, pre_link: sync::atomic::AtomicPtr<Node>) -> Self {
        Node {
            k,
            child,
            back_link,
            pre_link,
        }

    }
}

impl NodePtr {
    fn new(flag: bool, mark: bool, thread: bool, node_ref: Arc<Node> ) -> Self {
       NodePtr {
        flag,
        mark,
        thread,
        node_ref,
       }
    }
}

impl Bst {
    fn new(optimize : bool) -> Self {
        let first_root_left_child = Arc::new(Node::new(NEGATIVE_INF, [sync::atomic::AtomicPtr::new(core::ptr::null_mut()),sync::atomic::AtomicPtr::new(core::ptr::null_mut())], sync::atomic::AtomicPtr::new(core::ptr::null_mut()), sync::atomic::AtomicPtr::new(core::ptr::null_mut())));
        let first_root_right_child = Arc::new(Node::new(POSITIVE_INF, [sync::atomic::AtomicPtr::new(core::ptr::null_mut()), sync::atomic::AtomicPtr::new(core::ptr::null_mut())], sync::atomic::AtomicPtr::new(core::ptr::null_mut()), sync::atomic::AtomicPtr::new(core::ptr::null_mut())));
        let root = [first_root_left_child.clone(), first_root_right_child.clone()];
        root[0].child[0].store(Box::into_raw(Box::new(NodePtr::new(false, false, true, root[0].clone()))), sync::atomic::Ordering::SeqCst);
        root[0].child[1].store(Box::into_raw(Box::new(NodePtr::new(false, false, true, root[1].clone()))), sync::atomic::Ordering::SeqCst);
        root[0].back_link.store(Arc::into_raw(root[1].clone()) as * mut Node, sync::atomic::Ordering::SeqCst);
        root[0].pre_link.store(core::ptr::null_mut(), sync::atomic::Ordering::SeqCst);

        root[1].child[0].store(Box::into_raw(Box::new(NodePtr::new(false, false, true, root[0].clone()))), sync::atomic::Ordering::SeqCst);
        root[1].child[1].store(Box::into_raw(Box::new(NodePtr::new(false, false, true, unsafe { Arc::from_raw(ptr::null_mut()) }))), sync::atomic::Ordering::SeqCst);
        root[1].back_link.store(core::ptr::null_mut(), sync::atomic::Ordering::SeqCst);
        root[1].pre_link.store(core::ptr::null_mut(), sync::atomic::Ordering::SeqCst);
        
        Bst {
            root, 
            optimization: optimize
        }
    }

    fn contains(&self, k: i32) -> bool {

        let dir = self.locate(self.root[0].clone(), self.root[1].clone(), k);

        if dir == 2 {
            return true;
        }

        false
    }

    fn cas(&self, old_value: &sync::atomic::AtomicPtr<NodePtr>, new_value: *mut NodePtr, replacement: *mut NodePtr) -> bool {
        const TIMES: i32 = 3;
        let mut res = false;
        for i in 0..TIMES {
              match old_value.compare_exchange_weak(new_value, replacement, sync::atomic::Ordering::SeqCst, sync::atomic::Ordering::Relaxed) {
                 Ok(_) => {
                 res = true;
                 break;
               },
               Err(e) => { 
                let _x = unsafe { Arc::from_raw(e) };
                println!("cas Error old is {:?}, {:?}", old_value.load(sync::atomic::Ordering::SeqCst), unsafe { Arc::from_raw(old_value.load(sync::atomic::Ordering::SeqCst)) } );

                println!("cas Error new is {:?}, {:?}", new_value, unsafe { Arc::from_raw(new_value) } );
                continue;
               }
        }
      }
      
      res
    }

    fn add(&mut self, k: i32) -> bool {
        let child = [sync::atomic::AtomicPtr::new(core::ptr::null_mut()),sync::atomic::AtomicPtr::new(core::ptr::null_mut())];
        let back_link = sync::atomic::AtomicPtr::new(core::ptr::null_mut());
        let pre_link = sync::atomic::AtomicPtr::new(core::ptr::null_mut());
        let node = Arc::new(Node::new(k, child, back_link, pre_link));
        let curr = self.root[1].clone();
        let prev = self.root[0].clone();
        node.child[0].store(Box::into_raw(Box::new(NodePtr::new(false, false, true, node.clone()))), sync::atomic::Ordering::SeqCst);
        loop {
            let dir:u32 = self.locate(prev.clone(), curr.clone(), k);
            println!("in add function {:?}", dir);
            if dir >= 2 {
                return false;
            } else {
                let nxt = curr.child[dir as usize].load(sync::atomic::Ordering::SeqCst);
                let temp = unsafe { Arc::from_raw(nxt as *const NodePtr) };
                let node_ptr = Arc::new(NodePtr::new(false, false, true, temp.node_ref.clone()));
                node.child[1].store(Arc::into_raw(node_ptr) as * mut NodePtr, sync::atomic::Ordering::SeqCst);
                node.back_link.store(Arc::into_raw(curr.clone()) as *mut Node , sync::atomic::Ordering::SeqCst);
                let result = self.cas(&curr.child[dir as usize], curr.child[dir as usize].load(sync::atomic::Ordering::SeqCst), Box::into_raw(Box::new(NodePtr::new(false, false, false, node.clone()))));
                if result == true {
                    return true;
                }
            }

        }
    }

    fn locate(&self, mut prev: Arc<Node>, mut curr: Arc<Node>, k: i32) -> u32 {
        let mut steps = 0;
        loop {
            steps += 1;
            if steps >= 100 {
                return 3;
            }

            let dir = self.compare(k, curr.k);
            if dir == 2 {
                return dir;
            } else {
                let node_ptr = curr.child[dir as usize].load(sync::atomic::Ordering::SeqCst);
                // Temporarily increment the reference count of the object to ensure it is not freed while being used
                // let temp = unsafe { Arc::increment_strong_count(node_ptr as *const NodePtr) };
                // Since we want to avoid using unsafe, we will not dereference the raw pointer.
                // Instead, we will temporarily create an Arc from the raw pointer, which increments the reference count safely.
                // This allows us to access the `thread` field safely. We then clone the Arc if we need to keep it beyond this scope.
                let temp_arc = unsafe { Arc::from_raw(node_ptr) };
                if temp_arc.thread {
                    if dir == 0 {
                        // We must forget the temporary Arc to prevent decrementing the reference count.
                        // The reference count will be decremented elsewhere in the code where the Arc is managed.
                        std::mem::forget(temp_arc);
                        return dir;
                    }
                }
               

                prev = curr;

                // Since the previous attempt to clone still resulted in a bus error, we need to ensure that the pointer we are cloning is valid.
                // The bus error might be due to dereferencing a null or invalid pointer.
                // We should add a check to ensure that the pointer is not null before attempting to clone.
                if !node_ptr.is_null()  {
                    curr = temp_arc.node_ref.to_owned();
                } else {

                    return 3;
                    // Handle the null pointer case appropriately, possibly by logging an error or returning an error code.
                    // This is a placeholder for error handling logic.
                   // println!("Error: Attempted to clone a null pointer.");
                    // Assuming we have a way to return an error, we might do something like:
                    // return Err("Null pointer error");
                }
                // Decrement the reference count after cloning to maintain the correct count
                unsafe { Arc::decrement_strong_count(node_ptr as *const NodePtr) };
                println!("{:?}", prev)
            }
        }
    }
    fn compare(&self, k: i32, curr: i32) -> u32 {
        if k == curr {
            return 2;
        } else if k > curr {
            return 1;
        }
        0
    }


}

fn main() {
    let mut bst = Bst::new(true);
     bst.add(3);
     bst.add(2);

    println!("{:?}", bst.contains(1));
    println!("{:?}", bst.contains(2));

}