use std::{
    cmp::Ordering::{
        Less,
        Equal,
        Greater,
    },
    boxed::Box,
    mem::{
        size_of,
        take,
    }
};

type Ref<T> = Option<Box<T>>;

#[derive(Debug)]
enum Color { R, B }

#[derive(Debug)]
struct Node<V, K: PartialOrd> {
    chld:   (Ref<Self>, Ref<Self>),
    color:  Color,
    key:    K,
    val:    V,
}

impl<V, K: PartialOrd> Node<V, K> {
    fn new( val: V, key: K, color: Color ) -> Self {
        Self{ chld: (None, None), color, key, val }
    }

    fn insert( &mut self, val: V, key: K ) {
        match self.key.partial_cmp(&key) {
            Some(Less)      =>  self.chld.0 = Some(
                if let Some( mut chld ) = take( &mut self.chld.0 ) {
                    chld.insert( val, key );
                    chld
                } else {
                    Box::new( Self::new( val, key, Color::R ) )
                } ),
            Some(Greater)   =>  self.chld.1 = Some( 
                if let Some( mut chld ) = take( &mut self.chld.1 ) {
                    chld.insert( val, key );
                    chld
                } else {
                    Box::new( Self::new( val, key, Color::R ) )
                } ),
            Some(Equal)     => self.val = val,
            None            => {}
        };
    }
}

fn main() {
    let mut root = Node::<f32, i32>::new( 1., 2, Color::B );
    println!( "{:#?}", root );
    println!( "{}", size_of::<Node<(u16, u16, u16), i16>>() );
}
