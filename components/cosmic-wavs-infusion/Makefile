OUTPUT_DIR?=../../compiled


## wasi-build: building the WAVS wasi component(s)
wasi-build:
	@echo "Building component: cosmic-wavs-infusion-demo"
	@cargo component build --release; cargo fmt
	@mkdir -p $(OUTPUT_DIR)
	@cp ../../target/wasm32-wasip1/release/*.wasm $(OUTPUT_DIR)
.PHONY: wasi-build
