fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let grammar = std::path::PathBuf::from(&manifest_dir).join("grammar.lalrpop");

    lalrpop::Configuration::new()
        .set_in_dir(&manifest_dir)
        .set_out_dir(&out_dir)
        .process_file(&grammar)
        .unwrap();
}
