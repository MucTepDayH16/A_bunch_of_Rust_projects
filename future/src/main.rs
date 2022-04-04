#![allow(non_camel_case_types, irrefutable_let_patterns)]

use futures::{
    future::Future,
    task::{Context, Poll},
};
use std::pin::Pin;

enum stage_add2 {
    Start,
}

struct fut_add2 {
    stage: stage_add2,
    args: (usize, usize),
}

impl fut_add2 {
    fn new(x: usize, y: usize) -> Self {
        Self {
            stage: stage_add2::Start,
            args: (x, y),
        }
    }
}

impl Future for fut_add2 {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let stage_add2::Start = self.stage {
            return Poll::Ready(self.args.0 + self.args.1);
        }
        unreachable!()
    }
}

enum stage_add3 {
    Start,
    Stage0 { fut: fut_add2 },
    Stage1 { fut: fut_add2 },
}

struct fut_add3 {
    stage: stage_add3,
    args: (usize, usize, usize),
}

impl fut_add3 {
    fn new(x: usize, y: usize, z: usize) -> Self {
        Self {
            stage: stage_add3::Start,
            args: (x, y, z),
        }
    }
}

impl Future for fut_add3 {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let stage_add3::Start = self.stage {
            let fut = fut_add2::new(self.args.0, self.args.1);
            self.stage = stage_add3::Stage0 { fut };
        }
        if let stage_add3::Stage0 { ref mut fut } = self.stage {
            let t = futures::ready!(Pin::new(fut).poll(cx));
            let fut = fut_add2::new(t, self.args.2);
            self.stage = stage_add3::Stage1 { fut };
        }
        if let stage_add3::Stage1 { ref mut fut } = self.stage {
            let r = futures::ready!(Pin::new(fut).poll(cx));
            return Poll::Ready(r);
        }
        unreachable!()
    }
}

/*
async fn add2(x: usize, y: usize) -> usize {
    x + y
}

async fn add3(x: usize, y: usize, z: usize) -> usize {
    let t = add2(x, y).await;
    let r = add2(t, z).await;
    r
}
*/

fn main() {
    let x = futures::executor::block_on(fut_add3::new(3, 2, 1));
    println!("{}", x);
}
