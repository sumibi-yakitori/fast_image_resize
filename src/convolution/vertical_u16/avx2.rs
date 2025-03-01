use std::arch::x86_64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::Pixel;
use crate::simd_utils;

pub(crate) fn vert_convolution<T>(
    src_image: TypedImageView<T>,
    mut dst_image: TypedImageViewMut<T>,
    coeffs: Coefficients,
) where
    T: Pixel<Component = u16>,
{
    // native::vert_convolution(src_image, dst_image, coeffs);
    let (values, window_size, bounds_per_pixel) =
        (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard32::new(values);
    let coefficients_chunks = normalizer_guard.normalized_chunks(window_size, &bounds_per_pixel);

    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_u16(&src_image, dst_row, coeffs_chunk, &normalizer_guard);
        }
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn vert_convolution_into_one_row_u16<T>(
    src_img: &TypedImageView<T>,
    dst_row: &mut [T],
    coeffs_chunk: optimisations::CoefficientsI32Chunk,
    normalizer_guard: &optimisations::NormalizerGuard32,
) where
    T: Pixel<Component = u16>,
{
    let mut xx: usize = 0;
    let src_width = src_img.width().get() as usize * T::components_count();
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;
    let dst_components = T::components_mut(dst_row);

    /*
        |R    G    B   | |R    G    B   | |R    G   | - |B   | |R    G    B   | |R    G    B   | |R   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415| - |0001| |0203 0405 0607| |0809 1011 1213| |1415|

        Shuffle to extract 0-1 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0
        hi: -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract 2-3 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4
        hi: -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4

        Shuffle to extract 4-5 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8
        hi: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract 6-7 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
        hi: -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let shuffles = [
        _mm256_set_m128i(
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0),
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0),
        ),
        _mm256_set_m128i(
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4),
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4),
        ),
        _mm256_set_m128i(
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8),
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8),
        ),
        _mm256_set_m128i(
            _mm_set_epi8(
                -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
            ),
            _mm_set_epi8(
                -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
            ),
        ),
    ];

    let precision = normalizer_guard.precision();
    let initial = _mm256_set1_epi64x(1 << (precision - 1));
    let mut comp_buf = [0i64; 4];

    // 16 components in one register - 1 = 15
    while xx < src_width.saturating_sub(15) {
        // 16 components / 4 per register = 4 registers
        let mut sum = [initial; 4];

        for (s_row, &coeff) in src_img.iter_rows(y_start).zip(coeffs) {
            let components = T::components(s_row);
            let coeff_i64x4 = _mm256_set1_epi64x(coeff as i64);
            let source = simd_utils::loadu_si256(components, xx);
            for i in 0..4 {
                let comp_i64x4 = _mm256_shuffle_epi8(source, shuffles[i]);
                sum[i] = _mm256_add_epi64(sum[i], _mm256_mul_epi32(comp_i64x4, coeff_i64x4));
            }
        }

        for i in 0..4 {
            _mm256_storeu_si256((&mut comp_buf).as_mut_ptr() as *mut __m256i, sum[i]);
            let component = dst_components.get_unchecked_mut(xx + i * 2);
            *component = normalizer_guard.clip(comp_buf[0]);
            let component = dst_components.get_unchecked_mut(xx + i * 2 + 1);
            *component = normalizer_guard.clip(comp_buf[1]);
            let component = dst_components.get_unchecked_mut(xx + i * 2 + 8);
            *component = normalizer_guard.clip(comp_buf[2]);
            let component = dst_components.get_unchecked_mut(xx + i * 2 + 9);
            *component = normalizer_guard.clip(comp_buf[3]);
        }

        xx += 16;
    }

    if xx < src_width {
        // 16 components / 4 per register = 4 registers
        let mut sum = [initial; 4];
        let mut buf = [0u16; 16];

        for (s_row, &coeff) in src_img.iter_rows(y_start).zip(coeffs) {
            let components = T::components(s_row);
            for (i, &v) in components.get_unchecked(xx..).iter().enumerate() {
                buf[i] = v;
            }
            let coeff_i64x4 = _mm256_set1_epi64x(coeff as i64);
            let source = simd_utils::loadu_si256(&buf, 0);
            for i in 0..4 {
                let comp_i64x4 = _mm256_shuffle_epi8(source, shuffles[i]);
                sum[i] = _mm256_add_epi64(sum[i], _mm256_mul_epi32(comp_i64x4, coeff_i64x4));
            }
        }

        for i in 0..4 {
            _mm256_storeu_si256((&mut comp_buf).as_mut_ptr() as *mut __m256i, sum[i]);
            let component = buf.get_unchecked_mut(i * 2);
            *component = normalizer_guard.clip(comp_buf[0]);
            let component = buf.get_unchecked_mut(i * 2 + 1);
            *component = normalizer_guard.clip(comp_buf[1]);
            let component = buf.get_unchecked_mut(i * 2 + 8);
            *component = normalizer_guard.clip(comp_buf[2]);
            let component = buf.get_unchecked_mut(i * 2 + 9);
            *component = normalizer_guard.clip(comp_buf[3]);
        }
        for (i, v) in dst_components
            .get_unchecked_mut(xx..)
            .iter_mut()
            .enumerate()
        {
            *v = buf[i];
        }
    }
}
