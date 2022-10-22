mod general;
mod color;
mod demosaicing;

pub use general::*;
pub use color::*;
pub use demosaicing::*;

#[macro_export]
macro_rules! iters_to_vec {
    [$iter:ident $($body:tt)*] => {
        iters_to_vec!(@acc($iter) $($body)*)
    };
    [@acc($($x:tt)*) .. $fn:ident ( $($params:tt)* ) $($body:tt)*] => {
        iters_to_vec!(@acc($($x)* . $fn($($params)*)) $($body)*)
    };
    [@acc($($x:tt)*) [. $fn:ident ( $($params:tt)* ) $($cond:tt)* ] $($body:tt)*] => {
        if $($cond)* {
            iters_to_vec!(@acc($fn($($x)*, $($params)*)) $($body)*)
        } else {
            iters_to_vec!(@acc($($x)*) $($body)*)
        }
    };
    [@acc($($x:tt)*) . $fn:ident ( $($params:tt)* ) $($body:tt)*] => {
        iters_to_vec!(@acc($fn($($x)*, $($params)*)) $($body)*)
    };
    [@acc($($x:tt)*)] => {
        $($x)* . collect::<Vec<_>>()
    };
}

pub use iters_to_vec;