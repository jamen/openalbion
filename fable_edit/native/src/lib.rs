use std::fs::File;

use neon::prelude::*;

use fable::{Decode,Wad,WadHeader,WadEntry};
use chrono::NaiveDateTime;

declare_types! {
    pub class JsWadEntry for WadEntry {
        init(mut cx) {
            let id: f64 = cx.argument::<JsNumber>(0)?.value();
            let offset: f64 = cx.argument::<JsNumber>(1)?.value();
            let length: f64 = cx.argument::<JsNumber>(2)?.value();
            let path: String = cx.argument::<JsString>(3)?.value();
            let created: f64 = cx.argument::<JsNumber>(4)?.value();
            let accessed: f64 = cx.argument::<JsNumber>(5)?.value();
            let written: f64 = cx.argument::<JsNumber>(6)?.value();

            Ok(
                WadEntry {
                    id: id.floor() as u32,
                    offset: offset.floor() as u32,
                    length: length.floor() as u32,
                    path: path,
                    created: NaiveDateTime::from_timestamp(created.floor() as i64, 0),
                    accessed: NaiveDateTime::from_timestamp(accessed.floor() as i64, 0),
                    written: NaiveDateTime::from_timestamp(written.floor() as i64, 0),
                }
            )
        }

        method id(mut cx) {
            let this = cx.this();
            let id = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.id
            };
            Ok(cx.number(id as f64).upcast())
        }

        method offset(mut cx) {
            let this = cx.this();
            let offset = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.offset
            };
            Ok(cx.number(offset as f64).upcast())
        }

        method length(mut cx) {
            let this = cx.this();
            let length = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.length
            };
            Ok(cx.number(length as f64).upcast())
        }

        method path(mut cx) {
            let this = cx.this();
            let path = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.path.clone()
            };
            Ok(cx.string(&path).upcast())
        }

        method created(mut cx) {
            let this = cx.this();
            let created = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.created.timestamp()
            };
            Ok(cx.number(created as f64).upcast())
        }

        method accessed(mut cx) {
            let this = cx.this();
            let accessed = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.accessed.timestamp()
            };
            Ok(cx.number(accessed as f64).upcast())
        }

        method written(mut cx) {
            let this = cx.this();
            let written = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.written.timestamp()
            };
            Ok(cx.number(written as f64).upcast())
        }

        method created(mut cx) {
            let this = cx.this();
            let created = {
                let guard = cx.lock();
                let entry = this.borrow(&guard);
                entry.created.timestamp()
            };
            Ok(cx.number(created as f64).upcast())
        }
    }

    pub class JsWad for Wad {
        init(mut cx) {
            let file_path: String = cx.argument::<JsString>(0)?.value();

            let mut file = match File::open(&file_path) {
                Ok(file) => file,
                Err(_) => return cx.throw_error("Failed to open file."),
            };

            let wad = match Wad::decode(&mut file) {
                Ok(wad) => wad,
                Err(error) => return cx.throw_error(format!("Failed to decode file: {:?}", error)),
            };

            Ok(wad)
        }

        method entries_count(mut cx) {
            let len = {
                let this = cx.this();
                let guard = cx.lock();
                let wad = this.borrow(&guard);
                wad.entries.len()
            };
            Ok(cx.number(len as f64).upcast())
        }

        method entries(mut cx) {
            let this = cx.this();

            let entries_len = {
                let guard = cx.lock();
                let wad = this.borrow(&guard);
                wad.entries.len()
            };

            let page_start: usize = cx.argument::<JsNumber>(0)?.value().floor() as usize;
            let page_start = if page_start > 0 { page_start } else { 0 };

            let page_end: usize = cx.argument::<JsNumber>(1)?.value().floor() as usize;
            let page_end = if page_end <= entries_len { page_end } else { entries_len };

            let len = if page_start <= page_end { page_end - page_start } else { 0 };

            let js_entries = JsArray::new(&mut cx, len as u32);

            for i in 0..len {
                let (
                    id,
                    offset,
                    length,
                    path,
                    created,
                    accessed,
                    written,
                ) = {
                    let guard = cx.lock();
                    let entries = &this.borrow(&guard).entries;
                    let entry = &entries[page_start + i];
                    (
                        entry.id as f64,
                        entry.offset as f64,
                        entry.length as f64,
                        entry.path.clone(),
                        entry.created.timestamp() as f64,
                        entry.accessed.timestamp() as f64,
                        entry.written.timestamp() as f64,
                    )
                };

                let args: Vec<Handle<'_, JsValue>> = vec![
                    cx.number(id).upcast(),
                    cx.number(offset).upcast(),
                    cx.number(length).upcast(),
                    cx.string(path).upcast(),
                    cx.number(created).upcast(),
                    cx.number(accessed).upcast(),
                    cx.number(written).upcast()
                ];

                let js_entry: Handle<'_, JsValue> = JsWadEntry::new(&mut cx, args)?.upcast();

                js_entries.set(&mut cx, i as u32, js_entry);
            }

            Ok(js_entries.upcast())
        }
    }
}

register_module!(mut cx, {
    cx.export_class::<JsWad>("Wad")?;
    Ok(())
});