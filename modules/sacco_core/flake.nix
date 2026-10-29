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
    lez_core.inputs.logos-module-builder.follows = "logos-module-builder";
    lez_indexer_module.inputs.logos-module-builder.follows = "logos-module-builder";
  };

  outputs = inputs@{ logos-module-builder, sacco_programs, ... }:
    logos-module-builder.lib.mkLogosModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
      preConfigure = ''
        export SACCO_PROGRAMS_ROOT="${sacco_programs}"
      '';
    };
}
