extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/cpucycles.c")
        .compile("libcpucycles.a");
}