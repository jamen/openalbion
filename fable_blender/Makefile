all:
	rm -rf out
	mkdir -p out
	cargo build --release
	cp target/release/fable_blender.dll .

package:
	zip fable_blender.zip fable_blender.dll fable_blender.py