## [0.8.0] - 2022-03-23

- Added optimisation for convolution of U16x3 images with helps of ``SSE4.1``
  and ``AVX2`` instructions.
- Added partial optimisation for convolution of U8 images with helps of 
  ``SSE4.1`` instructions.
- Allowed to create an instance of `Image`, `ImageVew` and `ImageViewMut` 
  from a buffer larger than necessary 
  ([#5](https://github.com/Cykooz/fast_image_resize/issues/5)).
- Breaking changes:
  - Removed methods: `Image::from_vec_u32()`, `Image::from_slice_u32()`.
  - Removed error `InvalidBufferSizeError`.

## [0.7.0] - 2022-01-27

- Added support of new type of pixels `PixelType::U16x3`.
- Breaking changes:
  - Added variant `U16x3` into the enum `PixelType`.

## [0.6.0] - 2022-01-12

- Added optimisation of multiplying and dividing image by alpha channel with helps
  of ``SSE4.1`` instructions.
- Improved performance of dividing image by alpha channel without forced 
  SIMD instructions.
- Breaking changes:
  - Deleted variant `SSE2` from enum ``CpuExtensions``.

## [0.5.3] - 2021-12-14

- Added optimisation of convolution U8x3 images with helps of ``AVX2`` instructions.
- Fixed error in code for convolution U8x4 images with helps of ``SSE4.1`` instructions.
- Fixed error in code for convolution U8 images with helps of ``AVX2`` instructions.

## [0.5.2] - 2021-11-26

- Fixed compile errors on non-x86 architectures.

## [0.5.1] - 2021-11-24

- Fixed compile errors on non-x86 architectures.

## [0.5.0] - 2021-11-18

- Added support of new type of pixels `PixelType::U8x3` (with 
  auto-vectorization for SSE4.1).
- Exposed module `fast_image_resize::pixels` with types `U8x3`, 
  `U8x4`, `F32`, `I32`, `U8` used as wrappers for represent type of 
  one pixel of image.
- Some optimisations in code of convolution written in Rust (without 
  intrinsics for SIMD).
- Breaking changes:
  - Added variant `U8x3` into the enum `PixelType`.
  - Changed internal tuple structures inside of variant of `ImageRows` 
    and `ImageRowsMut` enums.

## [0.4.1] - 2021-11-13

- Added optimisation of convolution grayscale images (U8) with helps of ``AVX2`` instructions.

## [0.4.0] - 2021-10-23

- Added support of new type of pixels `PixelType::U8` (without forced SIMD).
- Breaking changes:
  - ``ImageData`` renamed into ``Image``.
  - ``SrcImageView`` and ``DstImageView`` replaced by ``ImageView``
    and ``ImageViewMut``.
  - Method ``Resizer.resize()`` now returns ``Result<(), DifferentTypesOfPixelsError>``.

## [0.3.1] - 2021-10-09

- Added support of compilation for architectures other than x86_64.

## [0.3.0] - 2021-08-28

- Added method `SrcImageView.set_crop_box_to_fit_dst_size()`.
- Fixed out-of-bounds error during resize with cropping.
- Refactored `ImageData`. 
  - Added methods: `from_vec_u32()`, `from_vec_u8()`, `from_slice_u32()`,
    `from_slice_u8()`.
  - Removed methods: `from_buffer()`, `from_pixels()`.

## [0.2.0] - 2021-08-02

- Fixed typo in name of CatmullRom filter type.
