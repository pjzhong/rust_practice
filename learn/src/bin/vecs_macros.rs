macro_rules! vecs {
    ( $($element:expr),* ) => {
       {
        // Enclose the xpansion in a block so that we use
        // multiple statements
        let mut v = Vec::new();

        // Start a repetition;
        $(
            // Each repeat will contain the follow statment,
            // with &element replaced with the corresponding expression
            v.push(format!("{}", $element));
        )*

         v
       }
    };
}

fn main() {
    let s = vecs![1, "a", true, 3.41414f32];
    assert_eq!(s, &["1", "a", "true", "3.41414f"])
}
