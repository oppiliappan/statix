pub mod main {
    use crate::err::StatixErr;

    use lib::LINTS;

    pub fn main() -> Result<(), StatixErr> {
        let mut lints = (*LINTS).clone();
        lints.as_mut_slice().sort_by_key(|a| a.code());
        for l in lints {
            println!("W{:02} {}", l.code(), l.name());
        }
        Ok(())
    }
}
