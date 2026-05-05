pub mod codegen;
pub mod parser;
pub mod tokenizer;

// for some reason the Try trait is still 'experimental', so in order to implement
// similiar behavior for TokenizerResult, I use this macro.
#[macro_export]
macro_rules! tryit {
    ($what:expr) => {{
        let val = $what;
        match val {
            TokenizerResult::Ok(val) => val,
            _ => return (val).coalesce_type(),
        }
    }};
}

#[derive(Debug)]
pub enum Attempt<T, E> {
    Ok(T),
    Err(E),
    Fallthrough,
}

#[allow(unused)]
impl<T, E> Attempt<T, E> {
    pub fn map<T2>(self, map: impl FnOnce(T) -> T2) -> Attempt<T2, E> {
        match self {
            Attempt::Ok(val) => Attempt::Ok(map(val)),

            _ => self.coalesce_type(),
        }
    }

    pub fn has_fallen(&self) -> bool {
        matches!(self, Self::Fallthrough)
    }

    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(val) => val,

            _ => panic!("TokenizerResult was not Ok"),
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    pub fn if_fell(self, map: impl FnOnce() -> Self) -> Self {
        if self.has_fallen() { map() } else { self }
    }

    pub fn coalesce_type<T2>(self) -> Attempt<T2, E> {
        match self {
            Attempt::Ok(_) => {
                panic!("TokenizerResult was Ok(_) which is invalid for this method.")
            }
            Attempt::Err(err) => Attempt::Err(err),
            Attempt::Fallthrough => Attempt::Fallthrough,
        }
    }
}