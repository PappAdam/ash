pub mod lin_alg;

#[macro_export]
#[allow(deref_nullptr)]
macro_rules! offset_of {
    { $type:ty, $field:tt } => {
        (&(*(0 as *const $type)).$field) as *const _ as usize
    };
}
