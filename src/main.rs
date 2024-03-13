use core::sync;
use std::{fmt, ptr};
//use std::ops::{ DerefMut, Deref };
use std::sync::Arc;

const POSITIVE_INF: u32 = u32::MAX;
const NEGATIVE_INF: u32 = u32::MIN;


#[derive(Debug)]
struct NodePtr {
    flag: bool,
    mark: bool,
    thread: bool,
    node_ref: Arc<Node>,
}

#[derive(Debug)]
struct Node {
    k: u32,
    child: [ sync::atomic::AtomicPtr<NodePtr>; 2],
    back_link: sync::atomic::AtomicPtr<Node>,
    pre_link: sync::atomic::AtomicPtr<Node>,
}

// impl DerefMut for Node {
//     fn deref_mut(&mut self) -> &mut Self {
//         self
//     }
// }

// impl Deref for Node {
//     type Target = Self;

//     fn deref(&self) -> &Self::Target {
//         self
//     }
// }

// impl Deref for NodePtr {
//     type Target = Self;

//     fn deref(&self) -> &Self::Target {
//         self
//     }
// }

// impl DerefMut for NodePtr {
//     fn deref_mut(&mut self) -> &mut Self {
//         self
//     }
// }


// impl Clone for Node {
//     fn clone(&self) -> Self {
//         *self
//     }
// }

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
    fn new(k: u32, child: [sync::atomic::AtomicPtr<NodePtr>; 2], back_link: sync::atomic::AtomicPtr<Node>, pre_link: sync::atomic::AtomicPtr<Node>) -> Self {
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
        root[0].child[1].store(Box::into_raw(Box::new(NodePtr::new(false, false, true, first_root_right_child.clone()))), sync::atomic::Ordering::SeqCst);
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

    fn contains(&self, k: u32) -> bool {

        let dir = self.locate(self.root[0].clone(), self.root[1].clone(), k);

        if dir == 2 {
            return true;
        }

        false
      
    }

    fn cas(&self, old_value: sync::atomic::AtomicPtr<NodePtr>, new_value: sync::atomic::AtomicPtr<NodePtr>, replacement: sync::atomic::AtomicPtr<NodePtr>) -> bool {

        const TIMES: i32 = 3;
        let mut res = false;
        for i in 0..TIMES {
              match old_value.compare_exchange_weak(new_value.load(sync::atomic::Ordering::SeqCst), replacement.load(sync::atomic::Ordering::SeqCst), sync::atomic::Ordering::SeqCst, sync::atomic::Ordering::Relaxed) {
                 Ok(_) => {
                 res = true;
                 break;
               },
               Err(_) => continue,
        }
      }
      
      res
    }

    fn add(&mut self, k: u32) -> bool {
        let child = [sync::atomic::AtomicPtr::new(core::ptr::null_mut()),sync::atomic::AtomicPtr::new(core::ptr::null_mut())];
        let back_link = sync::atomic::AtomicPtr::new(core::ptr::null_mut());
        let pre_link = sync::atomic::AtomicPtr::new(core::ptr::null_mut());
        let node = Arc::new(Node::new(k, child, back_link, pre_link));
        let curr = self.root[1].clone();
        let prev = self.root[0].clone();
        node.child[0].store(Box::into_raw(Box::new(NodePtr::new(false, false, true, node.clone()))), sync::atomic::Ordering::SeqCst);
        loop {
            let dir = self.locate(prev.clone(), curr.clone(), k);
            if dir == 2 {
                return false;
            } else {
                let nxt = curr.child[dir as usize].load(sync::atomic::Ordering::SeqCst);
                let temp = unsafe { Arc::from_raw(nxt as *const NodePtr) };
                let node_ptr = Arc::new(NodePtr::new(false, false, true, temp.node_ref.clone()));
                node.child[1].store(Arc::into_raw(node_ptr) as * mut NodePtr, sync::atomic::Ordering::SeqCst);
                node.back_link.store(Arc::into_raw(curr.clone()) as *mut Node , sync::atomic::Ordering::SeqCst);
                let result = self.cas(sync::atomic::AtomicPtr::new(nxt), sync::atomic::AtomicPtr::new(Box::into_raw(Box::new(NodePtr::new(false, false, true, temp.node_ref.clone())))), sync::atomic::AtomicPtr::new(Box::into_raw(Box::new(NodePtr::new(false, false, false, node.clone())))));
                if result == true {
                    return true;
                }



            }

        }
     


    }

    fn locate(&self, mut prev : Arc<Node>, mut curr: Arc<Node>, k: u32) -> u32 {

        loop {
            let dir = self.compare(k, curr.k);
            if dir == 2 {
                return dir;
            }
            else {
                let node = curr.child[dir as usize].load(sync::atomic::Ordering::SeqCst);
                let temp = unsafe { Arc::from_raw(node as *const NodePtr) };
                if temp.thread {
                    if dir == 0 {
                        return dir;
                    }
                }

                prev = curr;
                
                curr = temp.node_ref.clone();
                println!("{:?}", prev)
            }

        }

       // return  -1;

    }

    fn compare(&self, k: u32, curr: u32) -> u32 {
        if k == curr {
            return 2;
        } else if k > curr {
            return 1;
        }
        0
    }


}









fn main() {
    let bst = Bst::new(true);
    println!("???{:?}", bst);
    println!("++++++++++++++++{:?}", unsafe { Arc::clone(&(*bst.root[0].child[0].load(sync::atomic::Ordering::SeqCst)).node_ref) });
    println!("++++++++++++++++{:?}", &(*bst.root[0]));

}