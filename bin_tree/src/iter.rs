use std::{marker::PhantomData, ptr::NonNull};

#[derive(Clone, Copy)]
enum IterPhase {
    Left,
    PostLeft,
    Right,
    PostRight,
}

struct RawIter<K, V> {
    fwd: Option<NonNull<super::Node<K, V>>>,
    bck: Option<NonNull<super::Node<K, V>>>,
    phases: [IterPhase; 2],
}

impl<K, V> RawIter<K, V> {
    #[inline]
    fn new(ptr: Option<NonNull<super::Node<K, V>>>) -> Self {
        let mut fwd = ptr;
        while let Some(ptr) =
            fwd.and_then(|mut fwd| unsafe { fwd.as_mut() }.c[0].as_deref_mut())
        {
            fwd = Some(NonNull::from(ptr));
        }
        let mut bck = ptr;
        while let Some(ptr) =
            bck.and_then(|mut bck| unsafe { bck.as_mut() }.c[1].as_deref_mut())
        {
            bck = Some(NonNull::from(ptr));
        }
        Self {
            fwd,
            bck,
            phases: [IterPhase::Left, IterPhase::Right],
        }
    }

    unsafe fn next_fwd(&mut self) -> Option<(NonNull<K>, NonNull<V>)> {
        loop {
            let s = unsafe { self.fwd?.as_mut() };

            if let IterPhase::Left = self.phases[0] {
                if let Some(c0) = s.c[0].as_deref_mut() {
                    self.fwd = Some(NonNull::from(c0));
                    self.phases[0] = IterPhase::Left;
                    continue;
                } else {
                    self.phases[0] = IterPhase::PostLeft;
                }
            }

            if let IterPhase::PostLeft = self.phases[0] {
                if self.fwd == self.bck {
                    return None;
                } else {
                    self.phases[0] = IterPhase::Right;
                    let (k, v) = &mut s.kv;
                    return Some((NonNull::from(k), NonNull::from(v)));
                }
            }

            if let IterPhase::Right = self.phases[0] {
                if let Some(c1) = s.c[1].as_deref_mut() {
                    self.fwd = Some(NonNull::from(c1));
                    self.phases[0] = IterPhase::Left;
                    continue;
                } else {
                    self.phases[0] = IterPhase::PostRight;
                }
            }

            if let IterPhase::PostRight = self.phases[0] {
                let p = unsafe { (*s).p?.as_mut() };
                if self.fwd == p.c[0].as_deref().map(NonNull::from) {
                    self.phases[0] = IterPhase::PostLeft;
                } else {
                    self.phases[0] = IterPhase::PostRight;
                }
                self.fwd = Some(NonNull::from(p));
            }
        }
    }

    unsafe fn next_bck(&mut self) -> Option<(NonNull<K>, NonNull<V>)> {
        loop {
            let s = unsafe { self.bck?.as_mut() };

            if let IterPhase::Right = self.phases[1] {
                if let Some(c0) = s.c[1].as_deref_mut() {
                    self.bck = Some(NonNull::from(c0));
                    self.phases[1] = IterPhase::Right;
                    continue;
                } else {
                    self.phases[1] = IterPhase::PostRight;
                }
            }

            if let IterPhase::PostRight = self.phases[1] {
                if false {
                    return None;
                } else {
                    self.phases[1] = IterPhase::Left;
                    let (k, v) = &mut s.kv;
                    return Some((NonNull::from(k), NonNull::from(v)));
                }
            }

            if let IterPhase::Left = self.phases[1] {
                if let Some(c1) = s.c[0].as_deref_mut() {
                    self.bck = Some(NonNull::from(c1));
                    self.phases[1] = IterPhase::Right;
                    continue;
                } else {
                    self.phases[1] = IterPhase::PostLeft;
                }
            }

            if let IterPhase::PostLeft = self.phases[1] {
                let p = unsafe { (*s).p?.as_mut() };
                if self.bck == p.c[0].as_deref().map(NonNull::from) {
                    self.phases[1] = IterPhase::PostLeft;
                } else {
                    self.phases[1] = IterPhase::PostRight;
                }
                self.bck = Some(NonNull::from(p));
            }
        }
    }

    unsafe fn last(self) -> Option<(NonNull<K>, NonNull<V>)> {
        let mut s = self.fwd?.as_mut();
        while let Some(mut p) = s.p {
            s = unsafe { p.as_mut() };
        }

        while let Some(r) = s.c[1].as_deref_mut() {
            s = r;
        }

        let (k, v) = &mut s.kv;
        Some((NonNull::from(k), NonNull::from(v)))
    }
}

impl<K, V> Clone for RawIter<K, V> {
    fn clone(&self) -> Self {
        Self {
            fwd: self.fwd,
            bck: self.bck,
            phases: self.phases,
        }
    }
}

impl<K, V> Copy for RawIter<K, V> {}

macro_rules! iter_impl {
    (ref $Name:ident, $Item:ty, $toItem:expr) => {
        iter_impl! { mut $Name, $Item, $toItem }

        impl<'n, K, V> Clone for $Name<'n, K, V> {
            fn clone(&self) -> Self {
                Self {
                    raw: self.raw,
                    _phantom: PhantomData,
                }
            }
        }

        impl<'n, K, V> Copy for $Name<'n, K, V> {}
    };
    (mut $Name:ident, $Item:ty, $toItem:expr) => {
        #[repr(transparent)]
        pub struct $Name<'n, K, V> {
            raw: RawIter<K, V>,
            _phantom: PhantomData<$Item>,
        }

        impl<'n, K, V> $Name<'n, K, V> {
            #[allow(dead_code)]
            pub(super) fn new(ptr: Option<NonNull<super::Node<K, V>>>) -> Self {
                Self {
                    raw: RawIter::new(ptr),
                    _phantom: PhantomData,
                }
            }
        }

        impl<'n, K, V> Iterator for $Name<'n, K, V> {
            type Item = $Item;

            fn next(&mut self) -> Option<$Item> {
                let (k, v) = unsafe { self.raw.next_fwd() }?;
                Some(unsafe { $toItem(k, v) })
            }

            fn last(self) -> Option<$Item> {
                let (k, v) = unsafe { self.raw.last() }?;
                Some(unsafe { $toItem(k, v) })
            }

            fn min(mut self) -> Option<$Item> {
                self.next()
            }

            fn max(self) -> Option<$Item> {
                self.last()
            }
        }
    };
}

impl<'n, K, V> DoubleEndedIterator for Iter<'n, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (k, v) = unsafe { self.raw.next_bck() }?;
        Some(unsafe { (k.as_ref(), v.as_ref()) })
    }
}

iter_impl! { ref Iter, (&'n K, &'n V), |k: NonNull<K>, v: NonNull<V>| (k.as_ref(), v.as_ref()) }
iter_impl! { ref Keys, &'n K, |k: NonNull<K>, _| k.as_ref() }
iter_impl! { ref Values, &'n V, |_, v: NonNull<V>| v.as_ref() }

iter_impl! { mut IterMut, (&'n K, &'n mut V), |k: NonNull<K>, mut v: NonNull<V>| (k.as_ref(), v.as_mut()) }
iter_impl! { mut ValuesMut, &'n mut V, |_, mut v: NonNull<V>| v.as_mut() }
