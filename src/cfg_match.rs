#[macro_export]
macro_rules! cfg_match {
    () => {};
    (_ => { $($tt:tt)* }) => {
        $($tt)*
    };
    (
        $head_pattern:meta => { $($head_body:tt)* }
        $($rest:tt)*
    ) => {

        #[cfg($head_pattern)]
        $crate::cfg_match! { _ => { $($head_body)* } }

        #[cfg(not($head_pattern))]
        $crate::cfg_match! {
            $($rest)*
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn correct_one_selected() {
        crate::cfg_match! {
            any() => {
                panic!();
            }
            all() => {

            }
            any() => {
                panic!();
            }
        }
    }

    #[test]
    fn underscore() {
        crate::cfg_match! {
            _ => {}
        }
    }

    #[test]
    fn fallback() {
        crate::cfg_match! {
            any() => {
                panic!();
            }
            any() => {
                panic!();
            }
            _ => {}
        }
    }
}
