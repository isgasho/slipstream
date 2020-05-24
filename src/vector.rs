use core::marker::PhantomData;
use core::ptr;
use core::slice;
use core::ops::*;

use generic_array::{ArrayLength, GenericArray};
use typenum::marker_traits::Unsigned;

use crate::{inner, Vector};

macro_rules! bin_op_impl {
    ($name: ident, $tr: ident, $meth: ident, $tr_assign: ident, $meth_assign: ident) => {
        impl<B, R, S> $tr for $name<B, R, S>
        where
            R: inner::Repr<B> + $tr<Output = R> + Copy,
            S: ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            #[inline]
            fn $meth(self, rhs: Self) -> Self {
                let content = self.content.iter()
                    .zip(rhs.content.iter())
                    .map(|(a, b)| $tr::$meth(*a, *b))
                    .collect();
                Self {
                    content,
                    _props: PhantomData,
                }
            }
        }

        impl<B, R, S> $tr_assign for $name<B, R, S>
        where
            R: inner::Repr<B> + $tr_assign + Copy,
            S: ArrayLength<R>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn $meth_assign(&mut self, rhs: Self) {
                for (r, s) in self.content.iter_mut().zip(rhs.content.iter()) {
                    $tr_assign::$meth_assign(r, *s);
                }
            }
        }
    }
}

macro_rules! una_op_impl {
    ($name: ident, $tr: ident, $meth: ident) => {
        impl<B, R, S> $tr for $name<B, R, S>
        where
            R: inner::Repr<B> + $tr<Output = R> + Copy,
            S: Unsigned + ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Output = Self;
            #[inline]
            fn $meth(self) -> Self {
                let content = self.content
                    .iter()
                    .copied()
                    .map($tr::$meth)
                    .collect();
                Self {
                    content,
                    _props: PhantomData,
                }
            }
        }
    }
}

macro_rules! vector_impl {
    ($name: ident, $align: expr) => {
        #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(C, align($align))]
        pub struct $name<B, R, S>
        where
            R: inner::Repr<B>,
            S: ArrayLength<R>,
            S::ArrayType: Copy,
        {
            content: GenericArray<R, S>,
            _props: PhantomData<B>,
        }

        impl<B, R, S> Vector<B> for $name<B, R, S>
        where
            B: 'static,
            R: inner::Repr<B> + 'static,
            S: ArrayLength<R> + ArrayLength<B> + 'static,
            <S as ArrayLength<R>>::ArrayType: Copy,
        {
            type Lanes = S;
            #[inline]
            fn new(input: &[B]) -> Self {
                assert_eq!(
                    input.len(),
                    S::USIZE,
                    "Creating vector from the wrong sized slice",
                );
                unsafe {
                    let content = ptr::read(input.as_ptr().cast());
                    Self {
                        content,
                        _props: PhantomData,
                    }
                }
            }
        }

        impl<B, R, S> Deref for $name<B, R, S>
        where
            R: inner::Repr<B>,
            S: ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Target = [B];
            #[inline]
            fn deref(&self) -> &[B] {
                unsafe {
                    slice::from_raw_parts(
                        self.content.as_ptr().cast(),
                        S::USIZE,
                    )
                }
            }
        }

        impl<B, R, S> DerefMut for $name<B, R, S>
        where
            R: inner::Repr<B>,
            S: ArrayLength<R>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn deref_mut(&mut self) -> &mut [B] {
                unsafe {
                    slice::from_raw_parts_mut(
                        self.content.as_mut_ptr().cast(),
                        S::USIZE,
                    )
                }
            }
        }

        impl<B, R, S> Index<usize> for $name<B, R, S>
        where
            R: inner::Repr<B>,
            S: ArrayLength<R>,
            S::ArrayType: Copy,
        {
            type Output = B;
            #[inline]
            fn index(&self, idx: usize) -> &B {
                self.deref().index(idx)
            }
        }

        impl<B, R, S> IndexMut<usize> for $name<B, R, S>
        where
            R: inner::Repr<B>,
            S: ArrayLength<R>,
            S::ArrayType: Copy,
        {
            #[inline]
            fn index_mut(&mut self, idx: usize) -> &mut B {
                self.deref_mut().index_mut(idx)
            }
        }

        bin_op_impl!($name, Add, add, AddAssign, add_assign);
        bin_op_impl!($name, Sub, sub, SubAssign, sub_assign);
        bin_op_impl!($name, Mul, mul, MulAssign, mul_assign);
        bin_op_impl!($name, Div, div, DivAssign, div_assign);
        bin_op_impl!($name, Rem, rem, RemAssign, rem_assign);
        bin_op_impl!($name, BitAnd, bitand, BitAndAssign, bitand_assign);
        bin_op_impl!($name, BitOr, bitor, BitOrAssign, bitor_assign);
        bin_op_impl!($name, BitXor, bitxor, BitXorAssign, bitxor_assign);

        una_op_impl!($name, Neg, neg);
        una_op_impl!($name, Not, not);
    }
}

vector_impl!(Packed1, 1);
vector_impl!(Packed2, 2);
vector_impl!(Packed4, 4);
vector_impl!(Packed8, 8);
vector_impl!(Packed16, 16);
vector_impl!(Packed32, 32);
vector_impl!(Packed64, 64);
