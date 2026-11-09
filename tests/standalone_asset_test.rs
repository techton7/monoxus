#![cfg(feature = "manganis-embedded")]

use dioxus::prelude::*;

#[test]
fn test_standalone_tailwind_asset_embedded() {
    // Ensure CARGO_MANIFEST_DIR is unset so we are strictly verifying standalone execution behavior
    unsafe {
        std::env::remove_var("CARGO_MANIFEST_DIR");
    }

    let asset: Asset = asset!("/assets/tailwind.css");
    let resolved = asset.resolve();
    let resolved_str = resolved.to_string_lossy().into_owned();

    println!("Resolved standalone asset path: {}", resolved_str);

    // Verify virtual path format: "/assets/<hash>-tailwind.css"
    assert!(
        resolved_str.starts_with("/assets/"),
        "Resolved path must start with /assets/, got: {resolved_str}"
    );
    assert!(
        resolved_str.ends_with("-tailwind.css"),
        "Resolved path must end with -tailwind.css, got: {resolved_str}"
    );

    // Verify embedded in manganis registry
    let embedded_bytes = manganis::embedded_asset(&resolved_str);
    assert!(
        embedded_bytes.is_some(),
        "Asset bytes must be registered in embedded registry"
    );
    let bytes = embedded_bytes.unwrap();
    assert!(!bytes.is_empty(), "Embedded CSS bytes must not be empty");

    // Verify serve_asset returns 200 OK with correct Content-Type header
    let response = dioxus::asset_resolver::native::serve_asset(&resolved_str)
        .expect("serve_asset should succeed for embedded asset");
    assert_eq!(
        response.status(),
        200,
        "serve_asset status code must be 200 OK"
    );
    assert_eq!(
        response.headers()["Content-Type"],
        "text/css; charset=utf-8",
        "Content-Type must be text/css; charset=utf-8"
    );
    assert_eq!(
        response.body(),
        bytes,
        "Response body must match embedded bytes exactly"
    );

    // Verify public asset_path returns CannotRepresentAsPath for embedded assets
    let path_err = dioxus::asset_resolver::asset_path(&resolved_str);
    assert!(
        matches!(
            path_err,
            Err(dioxus::asset_resolver::AssetPathError::CannotRepresentAsPath)
        ),
        "Embedded assets cannot be represented as filesystem paths"
    );
}
