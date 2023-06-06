#![feature(generators, generator_trait)]

use std::{
    marker::PhantomData,
    mem::take,
    ops::{
        Deref, DerefMut, Fn, Generator,
        GeneratorState::{self, *},
    },
    pin::Pin,
};

struct Gen<F, A, T> {
    gen: F,
    state: Option<A>,
    _mark: PhantomData<T>,
}

impl<F, A, T> Gen<F, A, T>
where
    F: Fn(A) -> Option<(A, T)>,
{
    fn new(gen: F, state: A) -> Self {
        Self {
            gen,
            state: Some(state),
            _mark: PhantomData,
        }
    }
}

impl<F, A, T> Gen<F, A, T>
where
    F: Fn(A) -> Option<(A, T)>,
    A: Default,
{
    fn new_default(gen: F) -> Self {
        Self {
            gen,
            state: Some(A::default()),
            _mark: PhantomData,
        }
    }
}

impl<F, A, T> Iterator for Gen<F, A, T>
where
    F: Fn(A) -> Option<(A, T)>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let state = take(&mut self.state).and_then(&self.gen)?;
        self.state = Some(state.0);
        Some(state.1)
    }
}

struct GenV2<G, R> {
    gen: G,
    _state: PhantomData<R>,
}

impl<G, R> GenV2<G, R>
where
    G: Generator<R>,
{
    fn new(gen: G) -> Self {
        Self {
            gen,
            _state: PhantomData,
        }
    }
}

impl<G> Iterator for GenV2<G, ()>
where
    G: Generator<Return = ()> + Unpin,
{
    type Item = <G as Generator>::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.gen).resume(()) {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(()) => None,
        }
    }
}

fn main() {
    let gen = Gen::new(
        |(mut i, a)| {
            i += 1;
            Some(((i, a * i), a))
        },
        (0, 1),
    );
    println!("{:?}", gen.skip(20).take(5).collect::<Vec<u128>>());

    let gen = GenV2::new(|| {
        let mut acc = 1;
        for i in 1.. {
            yield acc;
            acc *= i;
        }
        return;
    });
    println!("{:?}", gen.skip(20).take(5).collect::<Vec<u128>>());
}
