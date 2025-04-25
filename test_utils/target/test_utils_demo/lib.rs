
        use std::collections::HashMap; // This import is used
        use std::io::Read; // This import is unused
        
        fn main() {
            let mut map = HashMap::new();
            map.insert("key", "value");
            println!("{:?}", map);
        }
        