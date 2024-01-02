pub mod utility {
    use std::collections::hash_map::DefaultHasher;
    use std::fs::File;
    use std::hash::{Hash, Hasher};
    use std::io;
    use std::io::BufRead;
    use std::path::Path;

    pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn s_hash(string: &str) -> i16 {
        let mut hasher = DefaultHasher::new();

        string.hash(&mut hasher);

        let hash_value = hasher.finish() as i16;

        hash_value
    }

    #[macro_export]
    macro_rules! hashed_tree_map {
    [ $( $y:literal ),* ] => {
            BTreeMap::from([$((unique_i16!($y), entities::Token::Pred),)*])
    }}
    #[macro_export]
    macro_rules! precedence_tree_map {
    { $( $x:literal = { $( $y:literal : $t:literal ),* } > { $( $z:literal ),* } < { $( $k:literal ),* }),* $(,)?  } => {
        {
            let mut outer_map = BTreeMap::new();
            $(
                let mut inner_map = BTreeMap::new();
                $(inner_map.insert(unique_i16!($y), entities::Precedence::Needs(unique_i16!($t)));)*
                $(inner_map.insert(unique_i16!($z), entities::Precedence::Greater);)*
                $(inner_map.insert(unique_i16!($k), entities::Precedence::Lesser);)*
                outer_map.insert(unique_i16!($x), inner_map);
            )*
            outer_map
        }
    }
}
}
