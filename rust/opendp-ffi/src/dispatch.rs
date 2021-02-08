// DISPATCH MACROS

// MAIN ENTRY POINT
/*
ONE TYPE ARG:
dispatch!(func, [(rt_type1, [u32, u64])], (arg1, arg2))
    EXPANDS TO
match rt_type1.descriptor().as_str() {
    stringify!( u32 ) => func::<u32>(arg1, arg2),
    stringify!( u64 ) => func::<u64>(arg1, arg2),
    rt_typeector => {
        ::std::rt::begin_panic_fmt(&{})
    }
}

TWO TYPE ARGS:
dispatch!(func, [(rt_type1, [u32, u64]), (rt_type1, [i32, i64])], (arg1, arg2))
    EXPANDS TO
match rt_type1.descriptor().as_str() {
    stringify!( u32 ) => match rt_type1.descriptor().as_str() {
        stringify!( i32 ) => func::<u32, i32>(arg1, arg2),
        stringify!( i64 ) => func::<u32, i64>(arg1, arg2),
        rt_type => {
            ::std::rt::begin_panic_fmt(&{})
        }
    },
    stringify!( u64 ) => match rt_type1.descriptor().as_str() {
        stringify!( i32 ) => func::<u64, i32>(arg1, arg2),
        stringify!( i64 ) => func::<u64, i64>(arg1, arg2),
        rt_type => {
            ::std::rt::begin_panic_fmt(&{})
        }
    },
    rt_type => {
        ::std::rt::begin_panic_fmt(&{})
    }
}

AND SO ON...
*/
// dispatch!(func, [(rt_type1, [u32, u64]), (rt_type2, [i32, i64]), (rt_type3, [f32, f64])], (arg1, arg2))
macro_rules! dispatch {
    ($function:ident, [$($rt_dispatch_types:tt),+], $args:tt) => {
        disp!($function, [$($rt_dispatch_types),+], (), $args)
    };
}

// BUILDING BLOCK, could be moved inside dispatch! with @prefix trick.
// disp!(func, [(rt_type1, [u32, u64]), (rt_type2, [i32, i64]), (rt_type3, [f32, f64])], (), (arg1, arg2))
// disp!(func, [(rt_type2, [i32, i64]), (rt_type3, [f32, f64])], (u32), (arg1, arg2))
// disp!(func, [(rt_type3, [f32, f64])], (u32, i32), (arg1, arg2))
// disp!(func, [], (u32, i32, f32), (arg1, arg2))
macro_rules! disp {
    ($function:ident, [$rt_dispatch_types_0:tt, $($rt_dispatch_types_n:tt),+], $type_args:tt,       $args:tt) => {
        disp_expand!($function, $rt_dispatch_types_0, [$($rt_dispatch_types_n),+], $type_args, $args)
    };
    ($function:ident, [$rt_dispatch_types_0:tt],                               $type_args:tt,       $args:tt) => {
        disp_expand!($function, $rt_dispatch_types_0, [],                          $type_args, $args)
    };
    ($function:ident, [],                                                      ($($type_arg:ty),+), ($($arg:expr),*)) => {
        $function::<$($type_arg),+>($($arg),*)
    };
}

// BUILDING BLOCK, could be moved inside dispatch! with @prefix trick.
// disp_expand!(func, (rt_type1, [u32, iu64]), [(rt_type2, [i32, i64]), (rt_type3, [f32, f64])], (), (arg1, arg2))
// disp_expand!(func, (rt_type2, [i32, i64]), [(rt_type3, [i32, i64])], (u32), (arg1, arg2))
// disp_expand!(func, (rt_type3, [f32, f64]), [], (u32, i32), (arg1, arg2))
macro_rules! disp_expand {
    ($function:ident, ($rt_type:expr, @primitives),              $rt_dispatch_types:tt, $type_args:tt, $args:tt) => {
        disp_expand!($function, ($rt_type, [u32, u64, i32, i64, f32, f64, bool, String, u8]), $rt_dispatch_types, $type_args, $args)
    };
    ($function:ident, ($rt_type:expr, @numbers),                 $rt_dispatch_types:tt, $type_args:tt, $args:tt) => {
        disp_expand!($function, ($rt_type, [u32, u64, i32, i64, f32, f64, u8]), $rt_dispatch_types, $type_args, $args)
    };
    ($function:ident, ($rt_type:expr, [$($dispatch_type:ty),+]), $rt_dispatch_types:tt, $type_args:tt, $args:tt) => {
        match $rt_type.id {
            $(x if x == std::any::TypeId::of::<$dispatch_type>() => disp_1!($function, $rt_dispatch_types, $type_args, $dispatch_type, $args)),+,
            _ => panic!("No match for concrete type {:?}/{}", $rt_type.id, $rt_type.descriptor)
        }
    };
}

// BUILDING BLOCK, could be moved inside dispatch! with @prefix trick.
// disp_1!(func, [(rt_type2, [i32, i64]), (rt_type3, [f32, f64])], u32, (), (arg1, arg2))
// disp_1!(func, [(rt_type3, [i32, i64])], i32, (u32), (arg1, arg2))
// disp_1!(func, [], f32, (u32, i32), (arg1, arg2))
macro_rules! disp_1 {
    ($function:ident, $rt_dispatch_types:tt, ($($type_arg:ty),+), $type_arg_n:ty, $args:tt) => {
        disp!($function, $rt_dispatch_types, ($($type_arg),+, $type_arg_n), $args)
    };
    ($function:ident, $rt_dispatch_types:tt, (),                  $type_arg_n:ty, $args:tt) => {
        disp!($function, $rt_dispatch_types, ($type_arg_n),                 $args)
    };
}
