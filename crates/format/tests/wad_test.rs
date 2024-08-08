mod util;

use format::WadHeader;
use std::fs;

#[test]
fn test_parse_wad_header() {
    let fable_path = util::fable_path().unwrap();
    let final_albion_wad_path = fable_path.join("data/Levels/FinalAlbion.wad");
    let final_albion_wad = fs::read(final_albion_wad_path).unwrap();
    let final_albion_wad = final_albion_wad.as_slice();

    let header = WadHeader::from_bytes(final_albion_wad);

    println!("{:#?}", header);
}
