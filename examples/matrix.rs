#![feature(test)]
extern crate test;

use std::mem;
use std::ops::Mul;
use std::time::Instant;
use core::arch::x86_64 as arch;

use array_init::array_init;
use impatient::prelude::*;
use multiversion::multiversion;
use rand::random;

const SIZE: usize = 512;
struct Matrix([[u32; SIZE]; SIZE]);

type V = u32x4;

impl Matrix {
    fn random() -> Self {
        Self(array_init(|_| {
            array_init(|_| {
                random()
            })
        }))
    }

    #[multiversion]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1")]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx")]
    #[clone(target = "[x86|x86_64]+sse+sse2+sse3+sse4.1+avx+avx2")]
    fn mult_simd(&self, rhs: &Matrix) -> Matrix {
        let mut output = [[0u32; SIZE]; SIZE];
        let mut column: [V; SIZE / V::LANES] = [Default::default(); SIZE / V::LANES];
        for x in 0..SIZE {
            // Do we want some kind of gather/stride way to load the vectors?
            // Anyway, as this is likely slower, we make sure to do the columns less often and
            // cache them for each corresponding rows, which load much faster
            for i in 0..SIZE {
                column[i / V::LANES][i % V::LANES] = rhs.0[i][x];
            }
            for y in 0..SIZE {
                unsafe {
                    let mut result = arch::_mm_setzero_si128();
                    for (c, r) in column.iter().zip(self.0[y].chunks_exact(V::LANES)) {
                        result = arch::_mm_add_epi32(result, arch::_mm_mullo_epi32(mem::transmute(*c), mem::transmute(V::new(r))));
                    }

                    output[y][x] = mem::transmute::<_, V>(result).iter().sum();
                }
            }
        }
        Matrix(output)
    }
}

impl Mul for &'_ Matrix {
    type Output = Matrix;
    fn mul(self, rhs: &Matrix) -> Matrix {
        let mut output = [[0u32; SIZE]; SIZE];
        for x in 0..SIZE {
            for y in 0..SIZE {
                for z in 0..SIZE {
                    output[y][x] = output[y][x].wrapping_add(self.0[y][z].wrapping_mul(rhs.0[z][x]));
                }
            }
        }
        Matrix(output)
    }
}

fn timed<R, F: FnOnce() -> R>(f: F) -> R {
    let now = Instant::now();
    let result = test::black_box(f());
    println!("took {:?}", now.elapsed());
    result
}

fn main() {
    let a = Matrix::random();
    let b = Matrix::random();
    let z = timed(|| &a * &b);
    let x = timed(|| a.mult_simd_default_version(&b));
    let w = timed(|| a.mult_simd(&b));
    //assert_eq!(z, w);
    /*
    if let Ok(sse) = Sse4_1::detect() {
        let w = timed(|| unsafe { mul_sse(sse, &a, &b) });
        //assert_eq!(z, w);
    }
    if let Ok(avx) = Avx2::detect() {
        let w = timed(|| unsafe { mul_avx(avx, &a, &b) });
        //assert_eq!(z, w);
    }
    */
}
