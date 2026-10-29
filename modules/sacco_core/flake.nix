{
  description = "Transparent SACCO core Logos module";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
    lez_core.url = "github:logos-blockchain/logos-execution-zone-module/0f4aa1b847c9438fbe257d9ffe5bfe57affea009";
    lez_indexer_module.url = "github:logos-blockchain/lez-indexer-module/87463b1173fc685dd40f789d5de013c2564519da";
    sacco_programs = {
      url = "path:../../programs";
      flake = false;
    };
    nixpkgs.follows = "logos-module-builder/nixpkgs";
    lez_core.inputs.logos-module-builder.follows = "logos-module-builder";
    lez_indexer_module.inputs.logos-module-builder.follows = "logos-module-builder";
  };

  outputs = inputs@{ logos-module-builder, sacco_programs, nixpkgs, ... }:
    let
      # The Logos module builds are currently targeted at x86_64-linux.
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      cargoVendor = pkgs.rustPlatform.importCargoLock {
        lockFile = "${sacco_programs}/sacco_ledger/ffi/Cargo.lock";
        outputHashes = {
          "lee_core-0.1.0" = "sha256-Tf2YnXIqbAmz2BnEk747WEOnAW6xOlszoe2mn85hRLU=";
        };
      };
      cargoHome = pkgs.runCommand "sacco-ffi-cargo-home" {} ''
        mkdir -p "$out"
        cat > "$out/config.toml" <<'EOF'
        [net]
        offline = true

        [source.crates-io]
        replace-with = "vendored-sources"

        [source."git+https://github.com/logos-blockchain/logos-execution-zone.git?tag=v0.3.1"]
        git = "https://github.com/logos-blockchain/logos-execution-zone.git"
        tag = "v0.3.1"
        replace-with = "vendored-sources"

        [source.vendored-sources]
        directory = "${cargoVendor}"
        EOF
      '';
    in
    logos-module-builder.lib.mkLogosModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
      preConfigure = ''
        export SACCO_PROGRAMS_ROOT="${sacco_programs}"
        export CARGO_HOME="${cargoHome}"
      '';
      extraNativeBuildInputs = [ cargoVendor ];
    };
}
