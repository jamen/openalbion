pub mod decode;
pub mod encode;

use crate::script::tng::Tng;

#[derive(Debug,PartialEq)]
pub struct Gtg {
    maps: Vec<Tng>
}