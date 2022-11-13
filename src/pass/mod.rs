mod color;
mod demosaicing;
mod general;

pub use color::*;
pub use demosaicing::*;
pub use general::*;

#[macro_export]
macro_rules! iters_to_vec {
    [$iter:ident $($body:tt)*] => {
        iters_to_vec!(@acc($iter) $($body)*)
    };

    // go to match accumulator 
    [@acc($($x:tt)*) [$target:expr] { $($rules:tt)* } $($body:tt)*] => {
        iters_to_vec!(@acc($($x)*) @body($($body)*) @match($target) @match_acc() $($rules)*)
    };

    // if accumulator
    [@acc($($x:tt)*) [. $fn:ident ( $($params:tt)* ) $($cond:tt)* ] $($body:tt)*] => {
        if $($cond)* {
            iters_to_vec!(@acc($fn($($x)*, $($params)*)) $($body)*)
        } else {
            iters_to_vec!(@acc($($x)*) $($body)*)
        }
    };

    [@acc($($x:tt)*) .. $fn:ident ( $($params:tt)* ) $($body:tt)*] => {
        iters_to_vec!(@acc($($x)* . $fn($($params)*)) $($body)*)
    };
    [@acc($($x:tt)*) . $fn:ident ( $($params:tt)* ) $($body:tt)*] => {
        iters_to_vec!(@acc($fn($($x)*, $($params)*)) $($body)*)
    };
    [@acc($($x:tt)*)] => {
        $($x)* . collect::<Vec<_>>()
    };

    // match accumulator
    [@acc($($x:tt)*) @body($($body:tt)*) @match($target:expr) @match_acc($($r:tt)*) $(,)? $p:pat => . $fn:ident ( $($params:tt)* ) $($rules:tt)* ] => {
        iters_to_vec!(@acc($($x)*) @body($($body)*) @match($target) @match_acc($($r)* $p => iters_to_vec!(@acc($fn($($x)*, $($params)*)) $($body)*),) $($rules)* )
    };
    [@acc($($x:tt)*) @body($($body:tt)*) @match($target:expr) @match_acc($($r:tt)*)] => {
        match $target {
            $($r)*
        }
    }
}

pub use iters_to_vec;
