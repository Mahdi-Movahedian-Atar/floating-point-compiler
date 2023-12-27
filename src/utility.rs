pub mod utility {
    use std::fs::File;
    use std::io;
    use std::io::BufRead;
    use std::path::Path;

    pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
        where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub const fn cs_hash(string: &str) -> i16 {
        let mut hash: i32 = 1;
        let mut j: i32 = 0;
        let length = string.as_bytes().len() as i32;

        while j < length {
            let i = j + string.as_bytes()[0] as i32;
            hash *= i;
            hash %= i16::MAX as i32;
            j += 1;
        }
        return hash as i16;
    }

    pub fn s_hash(string: &str) -> i16 {
        let mut hash: i32 = 1;
        let mut j: i32 = 0;
        let length = string.as_bytes().len() as i32;
        let max = i32::MAX / i16::MAX as i32;

        while j < length {
            let mut i = j + string.as_bytes()[0] as i32;
            i = max % i;
            hash *= i;
            hash %= i16::MAX as i32;
            j += 1;
        }
        return hash as i16;
    }

    #[macro_export] macro_rules! hashed_tree_map {
    [ $( $y:literal ),* ] => {
        BTreeMap::from([$((cs_hash($y), entities::Token::Pred),)*])
    }
}
}