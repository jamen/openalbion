all:
	cd ../ && \
	cargo clean --doc && \
	cargo doc --workspace --no-deps

	rm -rf out
	cp -r ../target/doc out

test-deploy: all
	netlify deploy --site defable.netlify.com --dir out

deploy: all
	netlify deploy --site defable.netlify.com --dir out --prod