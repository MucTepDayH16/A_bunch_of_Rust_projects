use std::{
    fmt::{Display, Formatter, Result},
    mem::take,
    ops::Deref,
    rc::Rc,
};

type Ref<T> = Option<Rc<T>>;

#[derive(Debug, Default)]
struct Node<T> {
    item: T,
    next: Ref<Node<T>>,
}

impl<T> PartialEq for &Node<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq((*self) as *const Node<T>, (*other) as *const Node<T>)
    }
}

#[derive(Debug, Clone)]
pub struct NodeIter<'a, T>(Option<&'a Node<T>>);

impl<'a, T> PartialEq for NodeIter<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Some(node1), Some(node2)) => node1 == node2,
            _ => false,
        }
    }
}

impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.0 {
            self.0 = match &node.next {
                Some(rf) => Some(rf.deref()),
                None => None,
            };
            Some(&node.item)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeIterFinite<'a, T>(Option<&'a Node<T>>, Option<&'a Node<T>>);

impl<'a, T> PartialEq for NodeIterFinite<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Some(node1), Some(node2)) => node1 == node2,
            _ => false,
        }
    }
}

impl<'a, T> Iterator for NodeIterFinite<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.0 {
            if let Some(last_node) = self.1 {
                if node == last_node {
                    return None;
                }
            }
            self.0 = match &node.next {
                Some(rf) => Some(rf.deref()),
                None => None,
            };
            Some(&node.item)
        } else {
            None
        }
    }
}

/// Single linked list, that could be linked to  other list
///
/// # Example #
///
/// ```
/// use std::ops::{Add, Mul};
///
/// use multi_list::list::MultiList;
///
/// let mut list_str = MultiList::new();
///
/// list_str.push(" world!").push("Hello,");
///
/// // [ head -> [1]Hello, -> [1] world! ]
/// println!("{}", list_str);
///
/// let mut list_i32 = MultiList::new();
///
/// list_i32.push(1).push(2).push(3).push(4);
///
/// // 10
/// println!("{}", list_i32.evaluate(i32::add, 0));
///
/// // 24
/// println!("{}", list_i32.evaluate(i32::mul, 1));
/// ```
#[derive(Debug)]
pub struct MultiList<T> {
    head: Ref<Node<T>>,
}

impl<T: Display> Display for MultiList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut rf = &self.head;

        write!(f, "[ head")?;
        while let Some(node) = rf {
            write!(f, " -> [{}]{}", Rc::strong_count(node), node.item)?;
            rf = &node.next;
        }
        write!(f, " ]")
    }
}

impl<T> MultiList<T> {
    #[inline]
    pub fn new() -> Self {
        Self { head: None }
    }

    #[inline]
    pub fn iter(&self) -> NodeIter<T> {
        NodeIter(match &self.head {
            Some(node) => Some(&node),
            None => None,
        })
    }

    #[inline]
    pub fn iter_to<'a>(
        &'a self,
        node: NodeIter<'a, T>,
    ) -> NodeIterFinite<'a, T> {
        NodeIterFinite(
            match &self.head {
                Some(node) => Some(&node),
                None => None,
            },
            node.0,
        )
    }

    #[inline]
    pub fn from_to<'a>(
        first: NodeIter<'a, T>,
        last: NodeIter<'a, T>,
    ) -> NodeIterFinite<'a, T> {
        NodeIterFinite(first.0, last.0)
    }

    #[inline]
    pub fn branch(&self) -> Self {
        Self {
            head: match &self.head {
                Some(rf) => Some(rf.clone()),
                None => None,
            },
        }
    }

    pub fn push(&mut self, item: T) -> &mut Self {
        self.head = Some(Rc::new(Node {
            item,
            next: take(&mut self.head),
        }));
        self
    }

    pub fn pop(&mut self) -> Option<T> {
        let rf = take(&mut self.head)?;
        let node = Rc::try_unwrap(rf).ok()?;

        self.head = node.next;
        Some(node.item)
    }

    pub fn evaluate<'a, U>(&'a self, fnc: fn(U, &'a T) -> U, mut def: U) -> U {
        let mut rf = &self.head;
        while let Some(node) = rf {
            def = fnc(def, &node.item);
            rf = &node.next;
        }
        def
    }

    pub fn common<'a>(&'a self, other: &'a Self) -> NodeIter<'a, T> {
        let mut rf0 = &self.head;
        while let Some(node0) = rf0 {
            if Rc::strong_count(node0) > 1 {
                let mut rf1 = &other.head;
                while let Some(node1) = rf1 {
                    if Rc::ptr_eq(node0, node1) {
                        return NodeIter(Some(node0.deref()));
                    }
                    rf1 = &node1.next;
                }
            }
            rf0 = &node0.next;
        }
        NodeIter(None)
    }
}

impl<T: PartialEq> MultiList<T> {
    pub fn find(&self, item: &T) -> NodeIter<T> {
        let mut rf = &self.head;
        while let Some(node) = rf {
            if node.item.eq(item) {
                return NodeIter(Some(node.deref()));
            } else {
                rf = &node.next;
            }
        }
        NodeIter(None)
    }
}
