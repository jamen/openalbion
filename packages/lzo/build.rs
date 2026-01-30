fn main() {
    cc::Build::new()
        .include("./include/minilzo/include/lzo")
        .include("./include/minilzo")
        .file("./include/minilzo/minilzo.c")
        .compile("minilzo");
}
