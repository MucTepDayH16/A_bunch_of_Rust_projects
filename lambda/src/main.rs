#![feature(unboxed_closures, fn_traits)]
#![allow(invalid_type_param_default)]

use std::{
    fmt,
    ops::Mul,
};

pub struct Lambda<Args, R> {
    func: Box<dyn FnOnce<Args, Output = R>>,
}

macro_rules! impl_lambda {
    (* $r:ident <- <- ) => {
        impl<$r> FnOnce<()> for Lambda<(), $r> {
            type Output = $r;

            extern "rust-call" fn call_once(self, _: ()) -> $r {
                self.func.call_once(())
            }
        }
    };
    (* $r:ident <- $($a:ident: $as:ident ,)* <- $($b:ident: $bs:ident ,)*) => {
        impl<$r: 'static, $($as: 'static ,)* $($bs: 'static ,)*> FnOnce<( $($as ,)* )>
        for Lambda<( $($as,)* $($bs,)*),$r> {
            type Output = Lambda<( $($bs,)* ), $r>;

            extern "rust-call" fn call_once(self, ( $($a ,)* ): ( $($as ,)* )) -> Self::Output {
                Self::Output{ func: Box::new(move |$($b: $bs,)*| -> $r { self.func.call_once(( $($a,)* $($b,)* )) } )}
            }
        }
    };
    ($($as:ident: $As:ident,)* ; ) => {
        impl_lambda!{* ReturnType <- $($as: $As,)* <-}

		impl<X: 'static, $($As: 'static,)* R: 'static> Mul<Lambda<($($As,)*), X>> for Lambda<(X,), R> {
			type Output = Lambda<($($As,)*), R>;

			fn mul(self, other: Lambda<($($As,)*), X>) -> Self::Output {
				Self::Output{ func: Box::new(move |$($as: $As,)*| self.func.call_once((other.func.call_once(($($as,)*)),)) )}
			}
		}

		impl<$($As,)* R> fmt::Debug for Lambda<($($As,)*), R> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				$( write!(f, "({:?}) -> ", std::any::type_name::<$As>())?; )*
				write!(f, "({:?})", std::any::type_name::<R>())
			}
		}
    };
    ($($as:ident: $As:ident,)* ; $b:ident: $B:ident, $($bs:ident: $Bs:ident,)*) => {
        impl_lambda!{* ReturnType <- $($as: $As,)* <- $b: $B, $($bs: $Bs,)*}
        impl_lambda!{$($as: $As,)* $b: $B, ; $($bs: $Bs,)*}
    };
    ($($as:ident: $As:ident),*) => {
        impl_lambda!{; $($as: $As,)*}
    };
}

impl_lambda! {}
impl_lambda! { a: A }
impl_lambda! { a: A, b: B }
impl_lambda! { a: A, b: B, c: C }
impl_lambda! { a: A, b: B, c: C, d: D }
impl_lambda! { a: A, b: B, c: C, d: D, e: E }
impl_lambda! { a: A, b: B, c: C, d: D, e: E, f: F }
impl_lambda! { a: A, b: B, c: C, d: D, e: E, f: F, g: G }
impl_lambda! { a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H }

fn add<A: std::ops::Add>() -> Lambda<(A, A), A::Output> {
    Lambda {
        func: Box::new(move |a, b| a + b),
    }
}

pub fn mul<A: std::ops::Mul>() -> Lambda<(A, A), A::Output> {
    Lambda {
        func: Box::new(move |a, b| a * b),
    }
}

pub fn sub<A: std::ops::Sub>() -> Lambda<(A, A), A::Output> {
    Lambda {
        func: Box::new(move |a, b| a - b),
    }
}

pub fn div<A: std::ops::Div>() -> Lambda<(A, A), A::Output> {
    Lambda {
        func: Box::new(move |a, b| a / b),
    }
}

pub fn map<F, I, T>() -> Lambda<(F, I), std::iter::Map<I::IntoIter, F>>
where
    I: IntoIterator,
    F: Fn(I::Item) -> T,
{
    Lambda {
        func: Box::new(move |f, i| i.into_iter().map(f)),
    }
}

fn main() {
    println!(
        "{:?}",
        map()(|x| x * x)(vec![1, 2, 3, 4, 5])().collect::<Vec<i32>>()
    );
    println!("{}", (add()(3) * add()(4))(5)());
}
