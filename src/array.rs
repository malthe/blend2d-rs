use core::{fmt, marker::PhantomData, ops, ptr, slice};

use crate::{
    bl_range,
    error::{errcode_to_result, Result},
    variant::WrappedBlCore,
};

#[repr(transparent)]
pub struct Array<T: ArrayType> {
    core: ffi::BLArrayCore,
    _pd: PhantomData<T>,
}

unsafe impl<T: ArrayType> WrappedBlCore for Array<T> {
    type Core = ffi::BLArrayCore;
}

impl<T: ArrayType> Array<T> {
    pub fn new() -> Self {
        Array {
            core: *Self::none(T::IMPL_IDX),
            _pd: PhantomData,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe { ffi::blArrayClear(self.core_mut()) };
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        unsafe { ffi::blArrayShrink(self.core_mut()) };
    }

    #[inline]
    pub fn reserve(&mut self, n: usize) {
        self.try_reserve(n).unwrap();
    }

    #[inline]
    pub fn try_reserve(&mut self, n: usize) -> Result<()> {
        unsafe { errcode_to_result(ffi::blArrayReserve(self.core_mut(), n)) }
    }

    #[inline]
    pub fn truncate(&mut self, n: usize) {
        unsafe { ffi::blArrayResize(self.core_mut(), n.min(self.len()), ptr::null()) };
    }

    #[inline]
    pub fn resize(&mut self, fill: &[T]) -> Result<()> {
        unsafe {
            errcode_to_result(ffi::blArrayResize(
                self.core_mut(),
                fill.len(),
                fill.as_ptr() as *const _,
            ))
        }
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Result<()> {
        unsafe { errcode_to_result(ffi::blArrayRemoveIndex(self.core_mut(), index)) }
    }

    #[inline]
    pub fn remove_range<R: ops::RangeBounds<usize>>(&mut self, range: R) -> Result<()> {
        unsafe { errcode_to_result(ffi::blArrayRemoveRange(self.core_mut(), &bl_range(range))) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { self.impl_().__bindgen_anon_1.__bindgen_anon_1.size as usize }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.impl_().capacity as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn extend_from_slice(&mut self, data: &[T]) -> Result<()> {
        unsafe {
            errcode_to_result(ffi::blArrayAppendView(
                self.core_mut(),
                data.as_ptr() as *const _,
                data.len(),
            ))
        }
    }

    #[inline]
    pub fn insert_from_slice(&mut self, index: usize, data: &[T]) -> Result<()> {
        unsafe {
            errcode_to_result(ffi::blArrayInsertView(
                self.core_mut(),
                index,
                data.as_ptr() as *const _,
                data.len(),
            ))
        }
    }

    #[inline]
    pub fn replace_from_slice<R: ops::RangeBounds<usize>>(
        &mut self,
        range: R,
        data: &[T],
    ) -> Result<()> {
        unsafe {
            errcode_to_result(ffi::blArrayReplaceView(
                self.core_mut(),
                &bl_range(range),
                data.as_ptr() as *const _,
                data.len(),
            ))
        }
    }

    #[inline]
    fn data_ptr(&self) -> *const T {
        unsafe { self.impl_().__bindgen_anon_1.__bindgen_anon_1.data as *const _ }
    }
}

impl<T: ArrayType> AsRef<[T]> for Array<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T: ArrayType> Borrow<[T]> for Array<T> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T: ArrayType> ops::Deref for Array<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.data_ptr(), self.len()) }
    }
}

impl<T, I> ops::Index<I> for Array<T>
where
    T: ArrayType,
    I: slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        ops::Index::index(&**self, index)
    }
}

impl<T: ArrayType> Default for Array<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ArrayType> PartialEq for Array<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { ffi::blArrayEquals(self.core(), other.core()) }
    }
}

impl<T: ArrayType> Clone for Array<T> {
    fn clone(&self) -> Self {
        Array {
            core: self.init_weak(),
            _pd: PhantomData,
        }
    }
}

impl<T: ArrayType> fmt::Debug for Array<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &*self)
    }
}

impl<T: ArrayType> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe { ffi::blArrayReset(&mut self.core) };
    }
}

impl<T> Array<T>
where
    T: ArrayType + WrappedBlCore,
{
    #[inline]
    pub fn push(&mut self, item: &T) -> Result<()> {
        unsafe {
            errcode_to_result(ffi::blArrayAppendItem(
                self.core_mut(),
                item.core() as *const _ as *const _,
            ))
        }
    }
    #[inline]
    pub fn insert(&mut self, index: usize, item: &T) -> Result<()> {
        unsafe {
            errcode_to_result(ffi::blArrayInsertItem(
                self.core_mut(),
                index,
                item.core() as *const _ as *const _,
            ))
        }
    }
    #[inline]
    pub fn replace(&mut self, index: usize, item: &T) -> Result<()> {
        unsafe {
            errcode_to_result(ffi::blArrayReplaceItem(
                self.core_mut(),
                index,
                item.core() as *const _ as *const _,
            ))
        }
    }
}

// Macro-zone ahead, you have been warned

macro_rules! impl_array_val_ops {
    ($( $append:ident, $insert:ident, $replace:ident for $($ty:ty),*);*$(;)*) => {
        $(
            $(
                impl Array<$ty> {
                    #[inline]
                    pub fn push(&mut self, item: $ty) -> Result<()> {
                        unsafe {
                            errcode_to_result(ffi::$append(
                                self.core_mut(),
                                item as _,
                            ))
                        }
                    }

                    #[inline]
                    pub fn insert(&mut self, index: usize, item: $ty) -> Result<()> {
                        unsafe {
                            errcode_to_result(ffi::$insert(
                                self.core_mut(),
                                index,
                                item as _,
                            ))
                        }
                    }

                    #[inline]
                    pub fn replace(&mut self, index: usize, item: $ty) -> Result<()> {
                        unsafe {
                            errcode_to_result(ffi::$replace(
                                self.core_mut(),
                                index,
                                item as _,
                            ))
                        }
                    }
                }
            )+
        )*
    }
}

impl_array_val_ops! {
    blArrayAppendU8,  blArrayInsertU8,  blArrayInsertU8   for i8,  u8;
    blArrayAppendU16, blArrayInsertU16, blArrayInsertU16  for i16, u16;
    blArrayAppendU32, blArrayInsertU32, blArrayInsertU32  for i32, u32;
    blArrayAppendU64, blArrayInsertU64, blArrayInsertU64  for i64, u64;
    blArrayAppendF32, blArrayInsertF32, blArrayInsertF32  for f32;
    blArrayAppendF64, blArrayInsertF64, blArrayInsertF64  for f64;
}

#[cfg(target_pointer_width = "32")]
impl_array_val_ops!(blArrayAppendU32, blArrayInsertU32, blArrayInsertU32 for isize, usize);
#[cfg(target_pointer_width = "64")]
impl_array_val_ops!(blArrayAppendU64, blArrayInsertU64, blArrayInsertU64 for isize, usize);

pub trait ArrayType: Sized {
    #[doc(hidden)]
    const IMPL_IDX: usize;
}

macro_rules! impl_array_type {
    ($(impl $($ty:ty),* = $idx:expr);*$(;)*) => {
        $(
            $(
                impl ArrayType for $ty {
                    const IMPL_IDX: usize = $idx as usize;
                }
            )+
        )*
    }
}

use ffi::BLImplType::*;
#[cfg(target_pointer_width = "32")]
const BL_IMPL_TYPE_ARRAY_USIZE: usize = BL_IMPL_TYPE_ARRAY_U32 as usize;
#[cfg(target_pointer_width = "32")]
const BL_IMPL_TYPE_ARRAY_ISIZE: usize = BL_IMPL_TYPE_ARRAY_I32 as usize;
#[cfg(target_pointer_width = "64")]
const BL_IMPL_TYPE_ARRAY_USIZE: usize = BL_IMPL_TYPE_ARRAY_U64 as usize;
#[cfg(target_pointer_width = "64")]
const BL_IMPL_TYPE_ARRAY_ISIZE: usize = BL_IMPL_TYPE_ARRAY_I64 as usize;

impl<T> ArrayType for *const T {
    const IMPL_IDX: usize = BL_IMPL_TYPE_ARRAY_USIZE;
}
impl<T> ArrayType for *mut T {
    const IMPL_IDX: usize = BL_IMPL_TYPE_ARRAY_USIZE;
}
impl<T> ArrayType for &T {
    const IMPL_IDX: usize = BL_IMPL_TYPE_ARRAY_USIZE;
}
impl<T> ArrayType for &mut T {
    const IMPL_IDX: usize = BL_IMPL_TYPE_ARRAY_USIZE;
}

impl_array_type! {
    impl i8 = BL_IMPL_TYPE_ARRAY_I8;
    impl u8 = BL_IMPL_TYPE_ARRAY_U8;
    impl i16 = BL_IMPL_TYPE_ARRAY_I16;
    impl u16 = BL_IMPL_TYPE_ARRAY_U16;
    impl i32 = BL_IMPL_TYPE_ARRAY_I32;
    impl u32 = BL_IMPL_TYPE_ARRAY_U32;
    impl i64 = BL_IMPL_TYPE_ARRAY_I64;
    impl u64 = BL_IMPL_TYPE_ARRAY_U64;
    impl isize = BL_IMPL_TYPE_ARRAY_ISIZE;
    impl usize = BL_IMPL_TYPE_ARRAY_USIZE;
    impl f32 = BL_IMPL_TYPE_ARRAY_F32;
    impl f64 = BL_IMPL_TYPE_ARRAY_F64;
}
use crate::{codec::ImageCodec, context::Context, image::Image, path::Path};
/* E0119, clashes with `impl<T> ArrayType for &T {[...]`
Specialization please (╯°□°）╯︵ ┻━┻
impl<T: WrappedBlCore> ArrayType for T {
  const IMPL_IDX: usize = BL_IMPL_TYPE_ARRAY_VAR as usize;
}
so we have to unfortunately go with a manual macro implementation */
impl_array_type! {
    impl Path, Image, ImageCodec, Context = BL_IMPL_TYPE_ARRAY_VAR;
}
use crate::Tag;
use std::borrow::Borrow;
impl_array_type! {
    impl Tag = BL_IMPL_TYPE_ARRAY_STRUCT_4;
}
