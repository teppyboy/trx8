fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rustc-link-arg-bin=trx8=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-bin=trx8=/MANIFESTUAC:level=\'requireAdministrator\'");
    }
}
