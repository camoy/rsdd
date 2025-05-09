use crate::{
    builder::{bdd::RobddBuilder, cache::AllIteTable, BottomUpBuilder},
    constants::primes,
    repr::{self, Cnf, DDNNFPtr, VarLabel, VarOrder, WmcParams},
    serialize::BDDSerializer,
    util::semirings::{Complex, FiniteField, RealSemiring, Semiring},
};
use std::{collections::HashMap, ffi::CStr, os::raw::c_char};

pub(super) type BddPtr = repr::BddPtr<'static>;

#[no_mangle]
extern "C" fn var_order_linear(num_vars: usize) -> *const VarOrder {
    Box::into_raw(Box::new(VarOrder::linear_order(num_vars)))
}

#[no_mangle]
unsafe extern "C" fn cnf_from_dimacs(dimacs_str: *const c_char) -> *const Cnf {
    let cstr = CStr::from_ptr(dimacs_str);

    Box::into_raw(Box::new(Cnf::from_dimacs(&String::from_utf8_lossy(
        cstr.to_bytes(),
    ))))
}

// directly inspired by https://users.rust-lang.org/t/how-to-deal-with-lifetime-when-need-to-expose-through-ffi/39583
// and the follow-up at https://users.rust-lang.org/t/can-someone-explain-why-this-is-working/82324/6
#[repr(C)]
struct RsddBddBuilder {
    _priv: [u8; 0],
}

unsafe fn robdd_builder_from_ptr<'_0>(
    ptr: *mut RsddBddBuilder,
) -> &'_0 mut RobddBuilder<'static, AllIteTable<BddPtr>> {
    if ptr.is_null() {
        eprintln!("Fatal error, got NULL `Context` pointer");
        ::std::process::abort();
    }
    &mut *(ptr.cast())
}

#[no_mangle]
unsafe extern "C" fn robdd_builder_all_table(order: *mut VarOrder) -> *mut RsddBddBuilder {
    if order.is_null() {
        eprintln!("Fatal error, got NULL `order` pointer");
        std::process::abort();
    }

    let order = *Box::from_raw(order);
    Box::into_raw(Box::new(RobddBuilder::<AllIteTable<BddPtr>>::new(order))).cast()
}

#[no_mangle]
unsafe extern "C" fn robdd_builder_compile_cnf(
    builder: *mut RsddBddBuilder,
    cnf: *mut Cnf,
) -> *mut BddPtr {
    if cnf.is_null() {
        eprintln!("Fatal error, got NULL `cnf` pointer");
        std::process::abort();
    }

    let builder = robdd_builder_from_ptr(builder);
    let cnf = *Box::from_raw(cnf);
    let ptr = builder.compile_cnf(&cnf);
    Box::into_raw(Box::new(ptr))
}

#[no_mangle]
unsafe extern "C" fn robdd_model_count(builder: *mut RsddBddBuilder, bdd: *mut BddPtr) -> u64 {
    let builder = robdd_builder_from_ptr(builder);
    let num_vars = builder.num_vars();
    let smoothed = builder.smooth(*bdd, num_vars);
    let unweighted_params: WmcParams<FiniteField<{ primes::U64_LARGEST }>> =
        WmcParams::new(HashMap::from_iter(
            (0..num_vars as u64)
                .map(|v| (VarLabel::new(v), (FiniteField::one(), FiniteField::one()))),
        ));

    let mc = smoothed.unsmoothed_wmc(&unweighted_params).value();
    mc as u64
}

// implementing the disc interface

#[no_mangle]
unsafe extern "C" fn mk_bdd_manager_default_order(num_vars: u64) -> *mut RsddBddBuilder {
    Box::into_raw(Box::new(RobddBuilder::<AllIteTable<BddPtr>>::new(
        VarOrder::linear_order(num_vars as usize),
    )))
    .cast()
}

#[no_mangle]
unsafe extern "C" fn bdd_new_label(builder: *mut RsddBddBuilder) -> u64 {
    let builder = robdd_builder_from_ptr(builder);
    builder.new_label().value()
}

#[no_mangle]
unsafe extern "C" fn bdd_var(
    builder: *mut RsddBddBuilder,
    label: u64,
    polarity: bool,
) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let ptr = builder.var(VarLabel::new(label), polarity);
    Box::into_raw(Box::new(ptr))
}

#[no_mangle]
unsafe extern "C" fn bdd_new_var(builder: *mut RsddBddBuilder, polarity: bool) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let (_, ptr) = builder.new_var(polarity);
    Box::into_raw(Box::new(ptr))
}

#[no_mangle]
unsafe extern "C" fn bdd_ite(
    builder: *mut RsddBddBuilder,
    f: *mut BddPtr,
    g: *mut BddPtr,
    h: *mut BddPtr,
) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let and = builder.ite(*f, *g, *h);
    Box::into_raw(Box::new(and))
}

#[no_mangle]
unsafe extern "C" fn bdd_and(
    builder: *mut RsddBddBuilder,
    left: *mut BddPtr,
    right: *mut BddPtr,
) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let and = builder.and(*left, *right);
    Box::into_raw(Box::new(and))
}

#[no_mangle]
unsafe extern "C" fn bdd_or(
    builder: *mut RsddBddBuilder,
    left: *mut BddPtr,
    right: *mut BddPtr,
) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let or = builder.or(*left, *right);
    Box::into_raw(Box::new(or))
}

#[no_mangle]
unsafe extern "C" fn bdd_negate(builder: *mut RsddBddBuilder, bdd: *mut BddPtr) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let negate = builder.negate(*bdd);
    Box::into_raw(Box::new(negate))
}

#[no_mangle]
unsafe extern "C" fn bdd_compose(
    builder: *mut RsddBddBuilder,
    f: *mut BddPtr,
    l: VarLabel,
    g: *mut BddPtr,
) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let composed = builder.compose(*f, l, *g);
    Box::into_raw(Box::new(composed))
}

#[no_mangle]
unsafe extern "C" fn bdd_is_true(bdd: *mut BddPtr) -> bool {
    (*bdd).is_true()
}

#[no_mangle]
unsafe extern "C" fn bdd_is_false(bdd: *mut BddPtr) -> bool {
    (*bdd).is_false()
}

#[no_mangle]
unsafe extern "C" fn bdd_is_const(bdd: *mut BddPtr) -> bool {
    (*bdd).is_const()
}

#[no_mangle]
unsafe extern "C" fn bdd_count_nodes(bdd: *mut BddPtr) -> usize {
    (*bdd).count_nodes()
}

#[no_mangle]
unsafe extern "C" fn bdd_scratch(bdd: *mut BddPtr, default: usize) -> usize {
    (*bdd).scratch::<usize>().unwrap_or(default)
}

#[no_mangle]
unsafe extern "C" fn bdd_set_scratch(bdd: *mut BddPtr, val: usize) {
    (*bdd).set_scratch::<usize>(val);
}

#[no_mangle]
unsafe extern "C" fn bdd_clear_scratch(bdd: *mut BddPtr) {
    (*bdd).clear_scratch();
}

#[no_mangle]
unsafe extern "C" fn bdd_true(builder: *mut RsddBddBuilder) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let bdd = builder.true_ptr();
    Box::into_raw(Box::new(bdd))
}

#[no_mangle]
unsafe extern "C" fn bdd_false(builder: *mut RsddBddBuilder) -> *mut BddPtr {
    let builder = robdd_builder_from_ptr(builder);
    let bdd = builder.false_ptr();
    Box::into_raw(Box::new(bdd))
}

#[no_mangle]
unsafe extern "C" fn bdd_eq(
    builder: *mut RsddBddBuilder,
    left: *mut BddPtr,
    right: *mut BddPtr,
) -> bool {
    let builder = robdd_builder_from_ptr(builder);
    builder.eq(*left, *right)
}

#[no_mangle]
unsafe extern "C" fn bdd_topvar(bdd: *mut BddPtr) -> u64 {
    match (*bdd).var_safe() {
        Some(x) => x.value(),
        None => 0, // TODO: fix this
    }
}

#[no_mangle]
unsafe extern "C" fn bdd_low(bdd: *mut BddPtr) -> *mut BddPtr {
    Box::into_raw(Box::new((*bdd).low()))
}

#[no_mangle]
unsafe extern "C" fn bdd_high(bdd: *mut BddPtr) -> *mut BddPtr {
    Box::into_raw(Box::new((*bdd).high()))
}

#[no_mangle]
unsafe extern "C" fn print_bdd(bdd: *mut BddPtr) -> *const c_char {
    let s = std::ffi::CString::new((*bdd).print_bdd()).unwrap();
    let p = s.as_ptr();
    std::mem::forget(s);
    p
}

#[no_mangle]
unsafe extern "C" fn bdd_num_recursive_calls(builder: *mut RsddBddBuilder) -> usize {
    let builder = robdd_builder_from_ptr(builder);
    builder.num_recursive_calls()
}

#[no_mangle]
pub unsafe extern "C" fn bdd_to_json(bdd: *mut BddPtr) -> *const c_char {
    let json = BDDSerializer::from_bdd(*bdd);
    let str = serde_json::to_string(&json).unwrap();
    let cstr = std::ffi::CString::new(str).unwrap();
    let p = cstr.as_ptr();
    std::mem::forget(cstr);
    p
}

#[no_mangle]
unsafe extern "C" fn bdd_wmc(bdd: *mut BddPtr, wmc: *mut WmcParams<RealSemiring>) -> f64 {
    DDNNFPtr::unsmoothed_wmc(&(*bdd), &(*wmc)).0
}

#[no_mangle]
unsafe extern "C" fn bdd_wmc_complex(bdd: *mut BddPtr, wmc: *mut WmcParams<Complex>) -> Complex {
    DDNNFPtr::unsmoothed_wmc(&(*bdd), &(*wmc))
}
