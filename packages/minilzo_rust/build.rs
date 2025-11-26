fn main() {
    cc::Build::new()
        .file("./include/minilzo/minilzo.c")
        .include("./include/minilzo")
        .include("./include/minilzo/include/lzo")
        .compile("minilzo");
}
