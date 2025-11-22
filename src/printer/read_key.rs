use console::{Key, Term};

/// Trait to allow reading keys from multiple inputs like [`Term`] (via [`Term::read_key`]) or a custom Testing utility.
pub trait ReadKey {
    fn read_key(&self) -> std::io::Result<Key>;
}

impl ReadKey for Term {
    fn read_key(&self) -> std::io::Result<Key> {
        self.read_key()
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::{cell::RefCell, io};

    use console::Key;

    use super::ReadKey;

    /// Test Utility to replay key sequences like otherwise gotten from a normal console via [`console::Term`].
    ///
    /// Will returns a error if the sequence has reached the end, but a new one is requested.
    #[derive(Debug, Clone)]
    pub struct TestKeys<'a> {
        data: &'a [Key],
        next_idx: RefCell<usize>,
    }

    impl<'a> TestKeys<'a> {
        pub fn new(data: &'a [Key]) -> Self {
            Self {
                data,
                next_idx: RefCell::new(0),
            }
        }

        /// Test if all the data in this instance has been replayed exactly.
        pub fn reached_end(&self) -> bool {
            *self.next_idx.borrow() == self.data.len()
        }
    }

    impl ReadKey for TestKeys<'_> {
        fn read_key(&self) -> io::Result<Key> {
            let idx = *self.next_idx.borrow();

            if idx >= self.data.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Reached the end of the data",
                ));
            }

            *self.next_idx.borrow_mut() += 1;
            Ok(self.data[idx].clone())
        }
    }
}
