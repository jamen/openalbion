fn main() {
    cc::Build::new()
        .file("./minilzo/minilzo.c")
        .include("./minilzo")
        .include("./minilzo/include/lzo")
        .compile("minilzo");
}
