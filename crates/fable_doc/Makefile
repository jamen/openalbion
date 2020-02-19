all:
	cd ../../ && \
	cargo clean --doc && \
	cargo doc --workspace --no-deps
	rm -rf out
	cp -r ../../target/doc out
	cp -r static/** out

deploy: all
	netlify deploy --site fable-doc --dir out