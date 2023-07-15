use std::{cmp::Ordering, fmt::Debug, iter::FromIterator, ptr::NonNull};

mod iter;

#[derive(Clone, Hash, PartialEq, Eq)]
struct Node<K, V> {
    kv: (K, V),
    c: [Ref<Self>; 2],
    p: Option<NonNull<Self>>,
}

impl<K: Debug, V: Debug> Debug for Node<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Node");
        s.field("key", &self.kv.0).field("value", &self.kv.1);
        if let Some(c0) = self.c[0].as_deref() {
            s.field("l", c0);
        }
        if let Some(c1) = self.c[1].as_deref() {
            s.field("r", c1);
        }
        s.finish()
    }
}

type Ref<T> = Option<Box<T>>;

impl<K, V> Node<K, V> {
    #[inline]
    fn root_p(kv: (K, V), p: Option<NonNull<Self>>) -> Self {
        Self {
            kv,
            c: [None, None],
            p,
        }
    }

    fn box_pair(self: Box<Self>) -> (K, V) {
        self.kv
    }

    #[allow(dead_code)]
    fn pair(self) -> (K, V) {
        self.kv
    }

    fn pair_ref(&self) -> (&K, &V) {
        let (k, v) = &self.kv;
        (k, v)
    }

    fn pair_mut(&mut self) -> (&K, &mut V) {
        let (k, v) = &mut self.kv;
        (&*k, v)
    }
}

impl<K: Ord, V> Node<K, V> {
    fn insert(
        maybe_this: &mut Ref<Self>,
        p: *mut Self,
        kv: (K, V),
    ) -> Option<(K, V)> {
        let Some(this) = maybe_this else {
            let this = Box::new(Node::root_p(kv, NonNull::new(p)));
            *maybe_this = Some(this);
            return None;
        };

        let p = this.as_mut() as _;
        match kv.0.cmp(&this.kv.0) {
            Ordering::Less => Self::insert(&mut this.c[0], p, kv),
            Ordering::Greater => Self::insert(&mut this.c[1], p, kv),
            Ordering::Equal => Some(std::mem::replace(&mut this.kv, kv)),
        }
    }

    fn find(&self, key: &K) -> Option<(&K, &V)> {
        match key.cmp(&self.kv.0) {
            Ordering::Less => self.c[0].as_deref()?.find(key),
            Ordering::Greater => self.c[1].as_deref()?.find(key),
            Ordering::Equal => Some(self.pair_ref()),
        }
    }

    fn find_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        match key.cmp(&self.kv.0) {
            Ordering::Less => self.c[0].as_deref_mut()?.find_mut(key),
            Ordering::Greater => self.c[1].as_deref_mut()?.find_mut(key),
            Ordering::Equal => Some(self.pair_mut()),
        }
    }

    fn remove(maybe_this: &mut Ref<Self>, key: &K) -> Option<(K, V)> {
        let this = maybe_this.as_deref_mut()?;
        match key.cmp(&this.kv.0) {
            Ordering::Less => Self::remove(&mut this.c[0], key),
            Ordering::Equal => Self::remove(&mut this.c[1], key),
            Ordering::Greater => {
                let lev = this.c[0].take();
                let prav = this.c[1].take();
                match (lev, prav) {
                    (None, mut child) | (mut child, None) => {
                        if let Some(child) = &mut child {
                            child.p = this.p;
                        }
                        std::mem::replace(maybe_this, child).map(Node::box_pair)
                    }
                    (Some(mut lev), Some(mut prav)) => {
                        let mut prav_lev_p = prav.as_mut() as *mut _;
                        let mut prav_lev = &mut prav.c[0];
                        while let Some(prav) = prav_lev {
                            prav_lev_p = prav.as_mut() as *mut _;
                            prav_lev = &mut prav.c[0];
                        }
                        lev.p = NonNull::new(prav_lev_p);
                        *prav_lev = Some(lev);
                        maybe_this.replace(prav).map(Node::box_pair)
                    }
                }
            }
        }
    }

    fn split(mut self: Box<Self>, key: &K) -> (Ref<Self>, Ref<Self>) {
        match key.cmp(&self.kv.0) {
            Ordering::Less => match self.c[0].take() {
                Some(l) => {
                    let (mut l0, l1) = l.split(key);
                    if let Some(l) = &mut l0 {
                        l.p = None;
                    }
                    self.set_child(true, l1);
                    (l0, Some(self))
                }
                None => (None, Some(self)),
            },
            Ordering::Greater => match self.c[1].take() {
                Some(r) => {
                    let (r0, mut r1) = r.split(key);
                    if let Some(r) = &mut r1 {
                        r.p = None;
                    }
                    self.set_child(false, r0);
                    (Some(self), r1)
                }
                None => (Some(self), None),
            },
            Ordering::Equal => {
                let mut r = self.c[1].take();
                if let Some(r) = &mut r {
                    r.p = None;
                }
                (Some(self), r)
            }
        }
    }

    fn set_child(&mut self, is_left: bool, mut c: Ref<Self>) {
        if let Some(c) = &mut c {
            c.p = Some(NonNull::from(&*self));
        }
        if is_left {
            self.c[0] = c;
        } else {
            self.c[1] = c;
        }
    }

    #[allow(dead_code)]
    fn rotate_left(self: &mut Box<Self>) {
        let Some(b) = self.c[1].take() else {
            return;
        };
        let mut a = std::mem::replace(self, b);
        self.p = a.p;
        let c = self.c[0].take();
        a.set_child(false, c);
        self.set_child(true, Some(a));
    }

    #[allow(dead_code)]
    fn rotate_right(self: &mut Box<Self>) {
        let Some(b) = self.c[0].take() else {
            return;
        };
        let mut a = std::mem::replace(self, b);
        self.p = a.p;
        let c = self.c[1].take();
        a.set_child(true, c);
        self.set_child(false, Some(a));
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct BinTree<K, V> {
    root: Ref<Node<K, V>>,
}

impl<K, V> Default for BinTree<K, V> {
    #[inline]
    fn default() -> Self {
        Self { root: None }
    }
}

impl<K: Debug + Ord, V: Debug> Debug for BinTree<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
        // self.root.fmt(f)
    }
}

impl<K, V> BinTree<K, V> {
    #[inline]
    fn new() -> Self {
        Self::default()
    }
}

impl<K: Ord, V> BinTree<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        Node::insert(&mut self.root, 0 as _, (key, value))
    }

    pub fn find(&self, key: &K) -> Option<(&K, &V)> {
        self.root.as_deref()?.find(key)
    }

    pub fn find_mut(&mut self, key: &K) -> Option<(&K, &mut V)> {
        self.root.as_deref_mut()?.find_mut(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<(K, V)> {
        Node::remove(&mut self.root, key)
    }

    pub fn iter(&self) -> iter::Iter<'_, K, V> {
        iter::Iter::new(self.root.as_deref().map(NonNull::from))
    }

    pub fn keys(&self) -> iter::Keys<'_, K, V> {
        iter::Keys::new(self.root.as_deref().map(NonNull::from))
    }

    pub fn values(&self) -> iter::Values<'_, K, V> {
        iter::Values::new(self.root.as_deref().map(NonNull::from))
    }

    pub fn iter_mut(&mut self) -> iter::IterMut<'_, K, V> {
        iter::IterMut::new(self.root.as_deref_mut().map(NonNull::from))
    }

    pub fn values_mut(&mut self) -> iter::ValuesMut<'_, K, V> {
        iter::ValuesMut::new(self.root.as_deref_mut().map(NonNull::from))
    }

    pub fn split(&mut self, key: &K) -> Self {
        match self.root.take() {
            Some(root) => {
                let (root0, root1) = root.split(key);
                self.root = root0;
                Self { root: root1 }
            }
            None => Self::new(),
        }
    }
}

impl<K: Ord, V> Extend<(K, V)> for BinTree<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K: Ord, V: Default> Extend<(K, Option<V>)> for BinTree<K, V> {
    fn extend<I: IntoIterator<Item = (K, Option<V>)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value.unwrap_or_default());
        }
    }
}

impl<K: Ord, V: Default> Extend<K> for BinTree<K, V> {
    fn extend<I: IntoIterator<Item = K>>(&mut self, iter: I) {
        for key in iter {
            self.insert(key, V::default());
        }
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for BinTree<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut new = Self::new();
        new.extend(iter);
        new
    }
}

impl<'n, K: Ord, V> IntoIterator for &'n BinTree<K, V> {
    type Item = (&'n K, &'n V);

    type IntoIter = iter::Iter<'n, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'n, K: Ord, V> IntoIterator for &'n mut BinTree<K, V> {
    type Item = (&'n K, &'n mut V);

    type IntoIter = iter::IterMut<'n, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

fn main() {
    let mut tree = BinTree::<_, usize>::new();
    tree.extend([8, 10, 14]);
    // println!("{:?}", tree.remove(&3));
    println!("{:#?}", tree);
    println!("{:?}", tree.find(&5));

    // let mut it = tree.iter();
    // println!("{:?}", it.next());
    // println!("{:?}", it.next());
    // println!("{:?}", it.next_back());
    // println!("{:?}", it.collect::<Vec<_>>());
}
