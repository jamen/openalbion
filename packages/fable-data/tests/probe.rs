use fable_data::def::binary::def_binary::{DefBinary, DefBody};
use fable_data::def::binary::names::Names;
use std::path::Path;

#[test]
#[ignore]
fn probe() {
    let dir = Path::new("/home/jamen/Fable/data/CompiledDefs");
    let names = Names::load(&dir.join("names.bin")).unwrap();
    let bin = DefBinary::load_with_names(&dir.join("game.bin"), &names).unwrap();
    let mut shown = 0;
    for e in bin.entries(&names) {
        if let DefBody::ThingObjectDef(d) = &e.record.body {
            let g = &d.graphic;
            // if our-layout reading looks sane (type small), it contradicts type-first
            println!("#{} {}: bank={} anim_bits={:#x} render={} alpha={} type={}",
                e.global_index, e.file_name.unwrap_or("?"), g.bank_index, g.anim_step.to_bits(), g.render_size_x, g.additive_alpha, g.type_);
            shown += 1;
            if shown >= 12 { break; }
        }
    }
}
