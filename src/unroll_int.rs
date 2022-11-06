macro_rules! create_unroll_int {
    // (_, 5) => 5
    (replace@ ($a:tt, $($b:tt)*)) => { $($b)* };

    // 2, 1, 0 => [0, 0, 0]
    (turn_into_zero_array@ $($num:literal)*) => {
        [$( create_unroll_int!(replace@ ($num, 0)) ),*]
    };

    ($first:literal $($rest:literal)* | $($acc:tt)*) => {
        create_unroll_int! {
            $($rest)*
            |
            ($first) => { create_unroll_int!(turn_into_zero_array@ $($rest)*) };
            $($acc)*
        }
    };

    (| $($acc:tt)*) => {
        macro_rules! unroll_int {
            $($acc)*
        }
    };

    ($($num:literal)*) => {
        create_unroll_int! { $($num)* | }
    };
}

create_unroll_int! {
    // 20 19 18 17 16 15 14 13 12 11
    //10 9 8 7 6 5 4 3
    2 1 0
}

fn x() {
    let x = unroll_int!(0);
}
