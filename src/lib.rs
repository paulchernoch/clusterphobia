extern crate serde;
extern crate csv;
extern crate hilbert;
pub mod clustering;

mod test_data;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
