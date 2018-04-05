#![cfg_attr(test, feature(generators, generator_trait))]
#![feature(pin)]

pub use pin_rc::*;
pub use pin_arc::*;

mod pin_rc;
mod pin_arc;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::{Generator, GeneratorState};
    use std::mem::Pin;

    trait SafeGenerator {
        type Yield;
        type Return;
        fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return>;
    }

    impl<'a, T: Generator + ?Sized> SafeGenerator for Pin<'a, T> {
        type Yield = T::Yield;
        type Return = T::Return;
        fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
            unsafe { Pin::get_mut(self).resume() }
        }
    }

    #[test]
    fn pin_rc_works() {
        let gen = PinRc::new(|| {
            for i in 0..10 {
                yield i;
            }
        });

        let mut results = Vec::new();
        while let GeneratorState::Yielded(x) = gen.borrow_mut().as_pin().resume() {
            results.push(x);
        }

        assert_eq!((0..10).collect::<Vec<_>>(), results);
    }

    #[test]
    fn pin_arc_works() {
        let gen = PinArc::new(|| {
            for i in 0..10 {
                yield i;
            }
        });

        let mut results = Vec::new();
        while let GeneratorState::Yielded(x) = gen.write().unwrap().as_pin().resume() {
            results.push(x);
        }

        assert_eq!((0..10).collect::<Vec<_>>(), results);
    }
}
