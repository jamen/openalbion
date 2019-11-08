pub mod decode;
pub mod encode;

use crate::tng::Tng;

#[derive(Debug,PartialEq)]
pub struct Gtg {
    maps: Vec<Tng>
}