pub mod decode;
pub mod encode;

use crate::Tng;

#[derive(Debug,PartialEq)]
pub struct Gtg {
    maps: Vec<Tng>
}