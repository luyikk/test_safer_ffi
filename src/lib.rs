use safer_ffi::prelude::*;
use safer_ffi::ptr;
use safer_ffi::ptr::NonNullOwned;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::rc::Rc;

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: f64,
    y: f64,
}

/* Export a Rust function to the C world. */
/// Returns the middle point of `[a, b]`.
#[ffi_export]
fn mid_point(a: &Point, b: &Point) -> Point {
    Point {
        x: (a.x + b.x) / 2.,
        y: (a.y + b.y) / 2.,
    }
}

/// Pretty-prints a point using Rust's formatting logic.
#[ffi_export]
fn print_point(point: &Point) {
    println!("{:?}", point);
}

#[ffi_export]
fn print(msg: char_p::Ref) -> char_p::Box {
    println!("{}", msg.to_str());
    format!("ok {}", msg).try_into().unwrap()
}

#[ffi_export]
fn drop_str(msg: char_p::Box) {
    drop(msg)
}

static MSG: &'static str = "123123";

#[ffi_export]
fn get_str() -> char_p::Ref<'static> {
    MSG.try_into().unwrap()
}

#[ffi_export]
fn call_closures(mut p: RefDynFnMut0<i32>) -> BoxDynFnMut1<i32, i32> {
    p.call();
    BoxDynFnMut1::new(Box::new(move |x| x))
}

#[ffi_export]
fn call_fun_ptr(p: extern "C" fn(a: i32, b: i32) -> i32) -> i32 {
    p(1, 2)
}

#[derive_ReprC]
#[ReprC::opaque]
pub struct ComplicatedStruct {
    path: PathBuf,
    cb: Rc<dyn 'static + Fn(&'_ Path)>,
    x: i32,
}

#[ffi_export]
fn create() -> repr_c::Box<ComplicatedStruct> {
    repr_c::Box::new(ComplicatedStruct {
        path: "/tmp".into(),
        cb: Rc::new(|path| println!("path = `{}`", path.to_string_lossy())),
        x: 42,
    })
}

#[ffi_export]
fn call_and_get_x(it: &'_ ComplicatedStruct) -> i32 {
    (it.cb)(&it.path);
    it.x
}

#[ffi_export]
fn destroy(it: repr_c::Box<ComplicatedStruct>) {
    drop(it)
}

#[ffi_export]
fn max(xs: c_slice::Ref<u8>) -> u8 {
    xs.as_slice() // : &'xs [i32]
        .iter()
        .max()
        .cloned()
        .map_or(0, |x| x)
}

#[derive_ReprC]
#[repr(transparent)]
pub struct Malloc<T>(ptr::NonNullOwned<T>);

impl<T> Malloc<T> {
    pub fn new(value: T) -> Option<Self> {
        Some(Self(NonNullOwned::from(
            NonNull::new(Box::into_raw(Box::new(value))).unwrap(),
        )))
    }
}

#[ffi_export]
fn new_int(x: i32) -> Option<Malloc<i32>> {
    Malloc::new(x)
}

#[ffi_export]
unsafe fn free_int(x:Malloc<i32>){
    x.0.drop_in_place_and_dealloc::<i32>()
}

// The following function is only necessary for the header generation.
#[cfg(feature = "headers")] // c.f. the `Cargo.toml` section
#[test]
pub fn generate_headers() -> ::std::io::Result<()> {
    ::safer_ffi::headers::builder()
        .to_file("rust_points.h")?
        .generate()
}

//             Rust	                    C
// Mutable pointer or NULL	    *mut T	Option<&mut T>
// Mutable pointer	*mut T	    &mut T
// Owned pointer or NULL	    *mut T	Option<repr_c::Box<T>>
// Owned pointer	*mut T	    repr_c::Box<T>



//              Rust	                                C
// cb: extern "C" fn()                              void (*cb)(void)
// f: extern "C" fn(arg1_t, arg2_t) -> ret_t	    ret_t (*f)(arg1_t, arg2_t)
// transmute::<_, extern "C" fn(arg_t) -> ret_t>(f)	(ret_t (*)(arg_t)) (f)
// type cb_t = extern "C" fn(arg_t) -> ret_t;
// let f: cb_t = ...;
// transmute::<_, cb_t>(f)	                        typedef ret_t (*cb_t)(arg_t);
// cb_t f = ...;
//                                                  (cb_t) (f)